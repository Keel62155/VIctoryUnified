use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const SUPPORTED_TIMESTAMP: u32 = 1_794_606_036;
const DLL_LOAD_DIR_AND_DEFAULT_DIRS: u32 = 0x1100;

#[repr(C)]
struct TitleInfo {
    struct_size: u32,
    version: u32,
    image_base: *mut c_void,
    image_size: u64,
    exe_path: *const u16,
}

const _: () = assert!(std::mem::size_of::<TitleInfo>() == 32);
type HostedStart = unsafe extern "C" fn(*const TitleInfo) -> i32;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryExW(path: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
    fn FreeLibrary(module: *mut c_void) -> i32;
}

fn folder() -> std::io::Result<PathBuf> {
    if let Some(dir) = std::env::var_os("VICTORY_SCRIPTHOOK_DIR").filter(|v| !v.is_empty()) {
        return PathBuf::from(dir).canonicalize();
    }
    let exe = std::env::current_exe()?;
    exe.with_file_name("..").join("scripthook").canonicalize()
}

fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain([0]).collect()
}

/// Start the locally installed trainer core once the supported image is mapped
/// and bound, before the title runs. When disabled, perform no filesystem work.
pub(crate) fn start(title_dir: &Path, base: usize, size: u64, timestamp: u32) {
    if std::env::var_os("VICTORY_SCRIPTHOOK").as_deref() != Some("1".as_ref()) {
        return;
    }
    if timestamp != SUPPORTED_TIMESTAMP {
        eprintln!("ScriptHook VI skipped: unsupported title timestamp {timestamp}.");
        return;
    }
    let paths = || -> std::io::Result<_> {
        let game = title_dir.canonicalize()?;
        let root = folder()?;
        let shvi = root.join("shvi").canonicalize()?;
        let dll = shvi.join("shvi_core.dll").canonicalize()?;
        if root.starts_with(&game) || !shvi.starts_with(&root) || !dll.starts_with(&shvi) {
            return Err(std::io::Error::other(
                "ScriptHook VI must be outside the game folder",
            ));
        }
        Ok((root, dll))
    };
    let (root, dll) = match paths() {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("ScriptHook VI skipped: {error}.");
            return;
        }
    };

    // SAFETY: the absolute path is nul-terminated and all loader arguments are valid.
    let module = unsafe {
        LoadLibraryExW(
            wide(&dll).as_ptr(),
            std::ptr::null_mut(),
            DLL_LOAD_DIR_AND_DEFAULT_DIRS,
        )
    };
    if module.is_null() {
        eprintln!(
            "ScriptHook VI could not load the local core: {}.",
            std::io::Error::last_os_error()
        );
        return;
    }
    // SAFETY: this is a live module and the export name is a C string.
    let entry = unsafe { GetProcAddress(module, c"victory_on_title_mapped".as_ptr().cast()) };
    if entry.is_null() {
        // SAFETY: no exported function has been called and this is our loader reference.
        unsafe { FreeLibrary(module) };
        eprintln!("ScriptHook VI skipped: the local core has no hosted startup export.");
        return;
    }

    // The fake title path selects the local shvi directory. Keep it for the
    // life of the process, including any deferred initialization by the core.
    let path = Box::leak(wide(&root.join("game_scarlett_master.exe")).into_boxed_slice());
    let info = TitleInfo {
        struct_size: std::mem::size_of::<TitleInfo>() as u32,
        version: 1,
        image_base: base as *mut c_void,
        image_size: size,
        exe_path: path.as_ptr(),
    };
    // SAFETY: the named export has the published hosted ABI. The caller keeps
    // the bound image mapped throughout execution. Keep the loaded DLL alive
    // because hosted startup may have started a worker or installed callbacks.
    let result = unsafe { std::mem::transmute::<*mut c_void, HostedStart>(entry)(&info) };
    if result == 0 {
        eprintln!(
            "ScriptHook VI accepted hosted startup. See the local shvi.log for initialization."
        );
    } else {
        eprintln!(
            "ScriptHook VI rejected hosted startup with code {result}. Continuing the title."
        );
    }
}
