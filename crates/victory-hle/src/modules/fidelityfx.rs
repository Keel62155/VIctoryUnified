//! `amd_fidelityfx_loader_xbox.dll`, redirected to a PC FidelityFX DLL.
//!
//! The Series S title upscales with AMD's FSR 4 through the FidelityFX API
//! (`ffxCreateContext`, `ffxDispatch`, ...). The console build and the PC
//! build export the same five functions. With `VICTORY_FSR_DLL=<path>` set
//! (`amd_fidelityfx_upscaler_dx12.dll`, or the `loader_dx12` beside it) the
//! title's loader is not mapped: its `LoadLibrary` gets this module, and
//! each call goes to the PC DLL, so tools that hook that DLL (OptiScaler)
//! see the title's upscaler like any PC game's.
//!
//! The PC DLL needs real D3D12 objects where the title passes its console
//! ones: the device in the context's backend description, and in a
//! dispatch the command list and the textures, which get the host list the
//! title's list records onto and the host textures behind the title's, in
//! the states the title declares. Descriptions are patched in place for the
//! call and restored after it.

#![allow(non_snake_case)]

use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use victory_gpu::external;

use crate::HleModule;

/// The environment variable that turns the redirect on.
const VARIABLE: &str = "VICTORY_FSR_DLL";

/// The title DLLs the redirect answers for.
const DLLS: [&str; 3] = [
    "amd_fidelityfx_loader_xbox.dll",
    "amd_fidelityfx_upscaler_xbox.dll",
    "amd_fidelityfx_framegeneration_xbox.dll",
];

fn path() -> Option<std::path::PathBuf> {
    std::env::var_os(VARIABLE)
        .filter(|p| !p.is_empty())
        .map(Into::into)
}

/// Whether the redirect is on.
pub fn enabled() -> bool {
    path().is_some()
}

/// Whether the title's `file` is answered by the redirect instead of being
/// mapped from the title's folder.
pub fn replaces(file: &str) -> bool {
    enabled() && DLLS.iter().any(|d| d.eq_ignore_ascii_case(file))
}

// ffxReturnCode_t
const OK: u32 = 0;
const ERROR: u32 = 1;
const ERROR_PARAMETER: u32 = 6;

// Description types (`ffx_api_types.h`, `ffx_api_dx12.h`, `ffx_upscale.h`).
const DESC_OVERRIDE_VERSION: u64 = 0x1;
const CREATE_BACKEND_DX12: u64 = 0x2;
const CREATE_UPSCALE: u64 = 0x1_0000;
const DISPATCH_UPSCALE: u64 = 0x1_0001;
const DISPATCH_GENERATE_REACTIVE_MASK: u64 = 0x1_0002;

// (From shuffl3r/victory-nvidia, GPL-3.0-or-later: the upscaler's descriptions,
// for DLSS Frame Generation through Streamline, `victory_gpu::streamline`.)

/// An upscaler's creation: flags and largest sizes.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct CreateUpscale {
    kind: u64,
    next: *mut u8,
    flags: u32,
    max_render: [u32; 2],
    max_output: [u32; 2],
    message: usize,
}

/// A resource, its description (type, format, sizes, mips, flags, usage)
/// and the state it is in.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Resource {
    resource: *mut c_void,
    description: [u32; 8],
    state: u32,
}

/// One upscaling: the command list, then colour, depth, motion vectors,
/// exposure, reactive mask, transparency and composition mask, output; then
/// jitter, motion-vector scale, sizes, sharpening, frame time, exposure,
/// reset, camera and flags. (The C `bool`s are read as bytes.)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct DispatchUpscale {
    kind: u64,
    next: *mut u8,
    list: *mut c_void,
    resources: [Resource; 7],
    jitter: [f32; 2],
    motion_scale: [f32; 2],
    render_size: [u32; 2],
    upscale_size: [u32; 2],
    enable_sharpening: u8,
    sharpness: f32,
    frame_time_delta: f32,
    pre_exposure: f32,
    reset: u8,
    camera_near: f32,
    camera_far: f32,
    camera_fov_angle_vertical: f32,
    view_space_to_meters_factor: f32,
    flags: u32,
}

const _: () = assert!(size_of::<CreateUpscale>() == 0x30);
const _: () = assert!(size_of::<Resource>() == 0x30);
const _: () = assert!(size_of::<DispatchUpscale>() == 0x1b0);

/// The upscaler contexts the title made: creation flags and largest output.
static CONTEXTS: Mutex<Vec<(usize, u32, [u32; 2])>> = Mutex::new(Vec::new());

fn created(context: usize) -> Option<(u32, [u32; 2])> {
    let contexts = CONTEXTS.lock().unwrap_or_else(|e| e.into_inner());
    contexts.iter().find(|c| c.0 == context).map(|c| (c.1, c.2))
}

/// An upscaling in `victory_gpu::streamline`'s terms (title objects, before
/// the description is patched with host ones).
fn upscale_of(d: &DispatchUpscale, flags: u32, max_output: [u32; 2], context: usize) -> victory_gpu::streamline::Upscale {
    let output = if d.upscale_size == [0, 0] { max_output } else { d.upscale_size };
    victory_gpu::streamline::Upscale {
        context,
        list: d.list.cast::<victory_gpu::object::Object>().cast_const(),
        resources: d.resources.map(|r| (r.resource.cast::<victory_gpu::object::Object>().cast_const(), r.state)),
        camera: victory_gpu::streamline::plan::Camera {
            jitter: d.jitter,
            motion_scale: d.motion_scale,
            render: d.render_size,
            output,
            near: d.camera_near,
            far: d.camera_far,
            fov_y: d.camera_fov_angle_vertical,
            reset: d.reset != 0,
            flags,
        },
        pre_exposure: d.pre_exposure,
        frame_ms: d.frame_time_delta,
    }
}

type Create = unsafe extern "system" fn(*mut *mut c_void, *mut u8, *const c_void) -> u32;
type Destroy = unsafe extern "system" fn(*mut *mut c_void, *const c_void) -> u32;
type Header = unsafe extern "system" fn(*mut *mut c_void, *mut u8) -> u32;

struct Api {
    create: Create,
    destroy: Destroy,
    configure: Header,
    query: Header,
    dispatch: Header,
}

/// The PC DLL's functions, loaded on first use.
fn api() -> Option<&'static Api> {
    static API: OnceLock<Option<Api>> = OnceLock::new();
    API.get_or_init(|| {
        let path = path()?;
        match load(&path) {
            Ok(api) => {
                println!("fidelityfx     {} (host, for the title's upscaler)", path.display());
                Some(api)
            }
            Err(error) => {
                tracing::error!(path = %path.display(), %error, "the PC FidelityFX DLL");
                None
            }
        }
    })
    .as_ref()
}

fn load(path: &std::path::Path) -> Result<Api, String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::System::LibraryLoader::{
        GetProcAddress, LOAD_WITH_ALTERED_SEARCH_PATH, LoadLibraryExW,
    };
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    // SAFETY: a NUL-terminated path. The DLL's own folder is searched for
    // what it needs.
    let module = unsafe { LoadLibraryExW(wide.as_ptr(), std::ptr::null_mut(), LOAD_WITH_ALTERED_SEARCH_PATH) };
    if module.is_null() {
        return Err(format!("LoadLibrary failed: {}", std::io::Error::last_os_error()));
    }
    let export = |name: &'static [u8]| {
        // SAFETY: a loaded module and a NUL-terminated name.
        unsafe { GetProcAddress(module, name.as_ptr()) }
            .map(|f| f as usize)
            .ok_or_else(|| format!("no export {}", String::from_utf8_lossy(&name[..name.len() - 1])))
    };
    // SAFETY: the five exports have these signatures in every FidelityFX
    // API DLL (`ffx_api.h`).
    unsafe {
        Ok(Api {
            create: std::mem::transmute::<usize, Create>(export(b"ffxCreateContext\0")?),
            destroy: std::mem::transmute::<usize, Destroy>(export(b"ffxDestroyContext\0")?),
            configure: std::mem::transmute::<usize, Header>(export(b"ffxConfigure\0")?),
            query: std::mem::transmute::<usize, Header>(export(b"ffxQuery\0")?),
            dispatch: std::mem::transmute::<usize, Header>(export(b"ffxDispatch\0")?),
        })
    }
}

/// Words of title memory changed for one call, put back after it.
#[derive(Default)]
struct Patches(Vec<(*mut usize, usize)>);

impl Patches {
    /// Sets the word at `at` to `new`.
    ///
    /// # Safety
    /// `at` is a writable, aligned word of a description the title passed.
    unsafe fn set(&mut self, at: *mut usize, new: usize) {
        // SAFETY: as the caller says.
        unsafe {
            self.0.push((at, at.read()));
            at.write(new);
        }
    }
}

impl Drop for Patches {
    fn drop(&mut self) {
        for &(at, old) in self.0.iter().rev() {
            // SAFETY: written by `set` on the same call.
            unsafe { at.write(old) };
        }
    }
}

/// Logs the description types a call carries, once per type: what the
/// title sends is not documented for the console build.
fn note(call: &str, head: *const u8) {
    static SEEN: Mutex<Vec<(String, u64)>> = Mutex::new(Vec::new());
    let mut node = head;
    for _ in 0..16 {
        if node.is_null() {
            break;
        }
        // SAFETY: a title description: type, then the next one.
        let (ty, next) = unsafe { (node.cast::<u64>().read_unaligned(), node.add(8).cast::<*const u8>().read_unaligned()) };
        let fresh = {
            let mut seen = SEEN.lock().unwrap_or_else(|e| e.into_inner());
            let key = (call.to_owned(), ty);
            let fresh = !seen.contains(&key);
            if fresh {
                seen.push(key);
            }
            fresh
        };
        if fresh {
            // SAFETY: the first 12 words of a description, as a dump to
            // learn its layout from.
            let words: Vec<u64> = (0..12).map(|i| unsafe { node.cast::<u64>().add(i).read_unaligned() }).collect();
            tracing::info!(call, ty = format_args!("{ty:#x}"), words = ?words.iter().map(|w| format!("{w:#x}")).collect::<Vec<_>>(), "FidelityFX description");
        }
        node = next;
    }
}

/// Replaces the title's device in the words `from..to` (byte offsets) of
/// description `node`.
fn swap_device(patches: &mut Patches, node: *mut u8, from: usize, to: usize, host: *mut c_void) {
    let mut at = from;
    while at + 8 <= to {
        // SAFETY: inside a description the title passed; read unaligned
        // first, patched only where it holds the title's device.
        unsafe {
            let word = node.add(at).cast::<usize>();
            if external::is_title_device(word.read_unaligned() as *const c_void) && word.is_aligned() {
                patches.set(word, host as usize);
            }
        }
        at += 8;
    }
}

/// `ffxCreateContext(context, desc, allocation callbacks)`: the backend
/// description names the host device; one is added if the title sends
/// none. The title's allocation callbacks are not passed on (the host DLL
/// allocates itself), nor is a version override: the title names a
/// provider of the console build.
extern "system" fn ffxCreateContext(context: *mut *mut c_void, desc: *mut u8, _alloc: *const c_void) -> u32 {
    let Some(api) = api() else { return ERROR };
    let Some(device) = external::host_device() else { return ERROR };
    note("create", desc);
    let mut patches = Patches::default();
    let mut backend = [CREATE_BACKEND_DX12, 0, device as u64];
    let mut has_backend = false;
    let mut last = desc;
    let mut previous: *mut u8 = std::ptr::null_mut();
    let mut node = desc;
    for _ in 0..32 {
        if node.is_null() {
            break;
        }
        // SAFETY: a title description: type, then the next one.
        let (ty, next) = unsafe { (node.cast::<u64>().read_unaligned(), node.add(8).cast::<*mut u8>().read_unaligned()) };
        if ty == CREATE_BACKEND_DX12 {
            has_backend = true;
            swap_device(&mut patches, node, 16, 24, device);
        }
        if ty == DESC_OVERRIDE_VERSION && !previous.is_null() {
            tracing::info!("FidelityFX: the title's version override is not passed to the host DLL");
            // SAFETY: the previous description's next-pointer word.
            unsafe { patches.set(previous.add(8).cast(), next as usize) };
        } else {
            previous = node;
        }
        last = node;
        node = next;
    }
    if !has_backend {
        tracing::info!("FidelityFX: no backend description from the title; adding the host device");
        // SAFETY: the last description's next-pointer word; `backend` lives
        // through the call.
        unsafe { patches.set(last.add(8).cast(), backend.as_mut_ptr() as usize) };
    }
    // SAFETY: the PC DLL's export, with a description the title wrote and
    // we patched.
    let code = unsafe { (api.create)(context, desc, std::ptr::null()) };
    drop(patches);
    // An upscaler: its flags and sizes, for frame generation's constants.
    // SAFETY: a description: its type first; an upscaler's by that type.
    if code == OK && !context.is_null() && unsafe { desc.cast::<u64>().read_unaligned() } == CREATE_UPSCALE {
        let c = unsafe { desc.cast::<CreateUpscale>().read_unaligned() };
        let handle = unsafe { *context } as usize;
        let mut contexts = CONTEXTS.lock().unwrap_or_else(|e| e.into_inner());
        contexts.retain(|x| x.0 != handle);
        contexts.push((handle, c.flags, c.max_output));
        tracing::info!(flags = format_args!("{:#x}", c.flags), max_render = ?c.max_render, max_output = ?c.max_output, "FidelityFX: the title made its upscaler");
    }
    tracing::info!(code, "ffxCreateContext on the host DLL");
    code
}

extern "system" fn ffxDestroyContext(context: *mut *mut c_void, _alloc: *const c_void) -> u32 {
    let Some(api) = api() else { return ERROR };
    if !context.is_null() {
        // SAFETY: the title's context.
        let handle = unsafe { *context } as usize;
        CONTEXTS.lock().unwrap_or_else(|e| e.into_inner()).retain(|x| x.0 != handle);
        victory_gpu::streamline::upscaler_destroyed(handle);
    }
    // SAFETY: the PC DLL's export.
    unsafe { (api.destroy)(context, std::ptr::null()) }
}

/// A description with the title's device somewhere in its first words
/// (queries and configurations): the host's in its place.
fn with_host_device(call: &str, context: *mut *mut c_void, desc: *mut u8, f: impl FnOnce(&Api) -> Header) -> u32 {
    let Some(api) = api() else { return ERROR };
    note(call, desc);
    let mut patches = Patches::default();
    if let Some(device) = external::host_device() {
        let mut node = desc;
        for _ in 0..16 {
            if node.is_null() {
                break;
            }
            swap_device(&mut patches, node, 16, 96, device);
            // SAFETY: a title description's next-pointer word.
            node = unsafe { node.add(8).cast::<*mut u8>().read_unaligned() };
        }
    }
    // SAFETY: the PC DLL's export.
    unsafe { f(api)(context, desc) }
}

extern "system" fn ffxConfigure(context: *mut *mut c_void, desc: *mut u8) -> u32 {
    with_host_device("configure", context, desc, |api| api.configure)
}

extern "system" fn ffxQuery(context: *mut *mut c_void, desc: *mut u8) -> u32 {
    with_host_device("query", context, desc, |api| api.query)
}

/// `FfxApiResourceState` to the D3D12 resource state it means.
fn d3d12_state(ffx: u32) -> u32 {
    const UNORDERED_ACCESS: u32 = 1 << 0;
    const COMPUTE_READ: u32 = 1 << 1;
    const PIXEL_READ: u32 = 1 << 2;
    const COPY_SRC: u32 = 1 << 3;
    const COPY_DEST: u32 = 1 << 4;
    const INDIRECT_ARGUMENT: u32 = 1 << 5;
    const RENDER_TARGET: u32 = 1 << 7;
    let mut state = 0;
    for (bit, d3d) in [
        (UNORDERED_ACCESS, 0x8),
        (COMPUTE_READ, 0x40),
        (PIXEL_READ, 0x80),
        (COPY_SRC, 0x800),
        (COPY_DEST, 0x400),
        (INDIRECT_ARGUMENT, 0x200),
        (RENDER_TARGET, 0x4),
    ] {
        if ffx & bit != 0 {
            state |= d3d;
        }
    }
    state
}

/// Where a dispatch description keeps its command list, and its resources
/// (each `FfxApiResource`: pointer, 32-byte description, state; 48 bytes)
/// with whether the effect writes them.
const RESOURCE_SIZE: usize = 48;
const RESOURCE_STATE: usize = 40;
const COMMAND_LIST: usize = 16;
const UPSCALE: [(usize, bool); 7] = [
    (24, false),
    (24 + RESOURCE_SIZE, false),
    (24 + 2 * RESOURCE_SIZE, false),
    (24 + 3 * RESOURCE_SIZE, false),
    (24 + 4 * RESOURCE_SIZE, false),
    (24 + 5 * RESOURCE_SIZE, false),
    (24 + 6 * RESOURCE_SIZE, true),
];
const REACTIVE_MASK: [(usize, bool); 3] = [
    (24, false),
    (24 + RESOURCE_SIZE, false),
    (24 + 2 * RESOURCE_SIZE, true),
];

/// `ffxDispatch(context, desc)`: the title's command list and textures
/// become the host's for the call.
extern "system" fn ffxDispatch(context: *mut *mut c_void, desc: *mut u8) -> u32 {
    let Some(api) = api() else { return ERROR };
    if desc.is_null() {
        return ERROR_PARAMETER;
    }
    note("dispatch", desc);
    // SAFETY: a description: its type first.
    let ty = unsafe { desc.cast::<u64>().read_unaligned() };
    let resources: &[(usize, bool)] = match ty {
        DISPATCH_UPSCALE => &UPSCALE,
        DISPATCH_GENERATE_REACTIVE_MASK => &REACTIVE_MASK,
        _ => {
            // SAFETY: the PC DLL's export; nothing here names console objects.
            return unsafe { (api.dispatch)(context, desc) };
        }
    };
    // SAFETY: the command list's word of a dispatch description.
    let list = unsafe { desc.add(COMMAND_LIST).cast::<usize>().read_unaligned() };
    // DLSS Frame Generation (`VICTORY_FRAMEGEN=1`): the upscaling's depth,
    // motion vectors and camera, read before the description is patched.
    let framegen = (ty == DISPATCH_UPSCALE && !context.is_null())
        .then(|| {
            // SAFETY: the title's context and an upscaling, by its type.
            let handle = unsafe { *context } as usize;
            let (flags, max_output) = created(handle)?;
            let d = unsafe { desc.cast::<DispatchUpscale>().read_unaligned() };
            Some(upscale_of(&d, flags, max_output, handle))
        })
        .flatten();
    let mut code = ERROR;
    let mut patches = Patches::default();
    let recorded = external::record(list as *const c_void, |pass| {
        if let Some(up) = &framegen {
            pass.note_framegen(up);
        }
        // SAFETY: the words of a dispatch description the title passed,
        // aligned as the C structures are.
        unsafe {
            patches.set(desc.add(COMMAND_LIST).cast(), pass.command_list() as usize);
            for &(at, write) in resources {
                let slot = desc.add(at).cast::<usize>();
                let title = slot.read_unaligned();
                if title == 0 {
                    continue;
                }
                let declared = desc.add(at + RESOURCE_STATE).cast::<u32>().read_unaligned();
                match pass.texture(title as *const c_void, d3d12_state(declared), write) {
                    Some(host) => patches.set(slot, host as usize),
                    None => {
                        tracing::warn!(slot = at, resource = title, "FidelityFX dispatch: not a texture of the title");
                        code = ERROR_PARAMETER;
                        return;
                    }
                }
            }
            code = (api.dispatch)(context, desc);
        }
    });
    drop(patches);
    if !recorded {
        tracing::warn!("FidelityFX dispatch: the command list is not one of the title's");
        return ERROR_PARAMETER;
    }
    if code != OK {
        tracing::warn!(code, "ffxDispatch on the host DLL");
    }
    code
}

pub(crate) fn module() -> HleModule {
    HleModule {
        name: "fidelityfx",
        dlls: &DLLS,
        exports: vec![
            crate::hle_export!(ffxCreateContext),
            crate::hle_export!(ffxDestroyContext),
            crate::hle_export!(ffxConfigure),
            crate::hle_export!(ffxQuery),
            crate::hle_export!(ffxDispatch),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_map_to_d3d12() {
        assert_eq!(d3d12_state(0), 0);
        assert_eq!(d3d12_state(1), 0x8);
        assert_eq!(d3d12_state((1 << 1) | (1 << 2)), 0xc0);
        assert_eq!(d3d12_state((1 << 3) | (1 << 1)), 0x840);
    }

    #[test]
    fn patches_are_put_back() {
        let mut words = [1usize, 2];
        {
            let mut p = Patches::default();
            // SAFETY: aligned words of a local array.
            unsafe {
                p.set(&mut words[0], 9);
                p.set(&mut words[0], 10);
                p.set(&mut words[1], 7);
            }
            assert_eq!(words, [10, 7]);
        }
        assert_eq!(words, [1, 2]);
    }
}
