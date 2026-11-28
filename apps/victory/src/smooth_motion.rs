//! `victory smooth-motion status|on|off`: NVIDIA Smooth Motion (the driver's own frame generation,
//! RTX 40 and 50 series, Direct3D 12) for `victory.exe`, through the driver's settings API (NVAPI
//! DRS) in `nvapi64.dll`.
//!
//! Only the two Smooth Motion settings of the driver profile that holds `victory.exe` are written:
//! `0xB0D384C0` (enable: 0 off, 1 on) and `0xB0CC0875` (the APIs it applies to: 1, Direct3D 12),
//! after making a profile "VIctoryUnified (victory.exe)" when none holds the executable. Nothing
//! else in the driver's settings is touched. The driver then shows a generated picture between two
//! of the title's: twice the pictures, the same responsiveness (only the title's frames take
//! input). Our own frame counter (the window title) counts the title's frames only.
//!
//! (Ported from AnyXboxX, LooperSalty/AnyXboxX `crates/anyxboxx-launcher/src/smooth_motion.rs`,
//! GPL-3.0-or-later; the setting and call ids are those NVIDIA Profile Inspector uses,
//! Orbmu2k/nvidiaProfileInspector.) A PowerShell version was blocked by the antivirus here (a
//! script that loads a DLL and calls into it by address), so it lives in this executable.

use std::ffi::c_void;

use anyhow::{Result, anyhow, bail};

/// The executable the setting is for (every `victory.exe`, whatever its folder: the driver's
/// profiles go by file name).
const EXE: &str = "victory.exe";
/// The driver profile made when none holds the executable.
const PROFILE: &str = "VIctoryUnified (victory.exe)";

const SM_ENABLE: u32 = 0xB0D3_84C0;
const SM_APIS: u32 = 0xB0CC_0875;
const APIS_DX12: u32 = 1;

const OK: i32 = 0;
const EXECUTABLE_NOT_FOUND: i32 = -166;
const PROFILE_NOT_FOUND: i32 = -163;
const SETTING_NOT_FOUND: i32 = -160;

const UNICODE_MAX: usize = 2048;
const VALUE_BYTES: usize = 4100;

#[repr(C)]
struct Profile {
    version: u32,
    name: [u16; UNICODE_MAX],
    gpu_support: u32,
    is_predefined: u32,
    num_of_apps: u32,
    num_of_settings: u32,
}

#[repr(C)]
struct Application {
    version: u32,
    is_predefined: u32,
    app_name: [u16; UNICODE_MAX],
    user_friendly_name: [u16; UNICODE_MAX],
    launcher: [u16; UNICODE_MAX],
    file_in_folder: [u16; UNICODE_MAX],
    bits: u32,
    command_line: [u16; UNICODE_MAX],
}

#[repr(C)]
struct Setting {
    version: u32,
    name: [u16; UNICODE_MAX],
    id: u32,
    kind: u32,
    location: u32,
    is_current_predefined: u32,
    is_predefined_valid: u32,
    predefined_value: [u8; VALUE_BYTES],
    current_value: [u8; VALUE_BYTES],
}

// NVAPI's structure versions: the size in the low 16 bits, the version above.
const PROFILE_VER: u32 = size_of::<Profile>() as u32 | (1 << 16);
const APPLICATION_VER: u32 = size_of::<Application>() as u32 | (4 << 16);
const SETTING_VER: u32 = size_of::<Setting>() as u32 | (1 << 16);

type Session = *mut c_void;
type Handle = *mut c_void;
type QueryInterface = unsafe extern "C" fn(u32) -> *mut c_void;
type NoArgs = unsafe extern "C" fn() -> i32;
type CreateSession = unsafe extern "C" fn(*mut Session) -> i32;
type OnSession = unsafe extern "C" fn(Session) -> i32;
type FindApp = unsafe extern "C" fn(Session, *const u16, *mut Handle, *mut Application) -> i32;
type FindProfile = unsafe extern "C" fn(Session, *const u16, *mut Handle) -> i32;
type CreateProfile = unsafe extern "C" fn(Session, *mut Profile, *mut Handle) -> i32;
type CreateApp = unsafe extern "C" fn(Session, Handle, *mut Application) -> i32;
type SetSetting = unsafe extern "C" fn(Session, Handle, *mut Setting, u32, u32) -> i32;
type GetSetting = unsafe extern "C" fn(Session, Handle, u32, *mut Setting, *mut u32) -> i32;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(name: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
}

/// A zeroed value of a plain-data NVAPI structure, on the heap (they run to 20 KB).
fn zeroed<T>() -> Box<T> {
    // SAFETY: every NVAPI structure here is integers and integer arrays, for which all-zero bytes
    // are a valid value.
    unsafe { Box::new(std::mem::zeroed()) }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn put(into: &mut [u16; UNICODE_MAX], text: &str) {
    for (to, from) in into.iter_mut().zip(text.encode_utf16().take(UNICODE_MAX - 1)) {
        *to = from;
    }
}

fn check(status: i32, what: &str) -> Result<()> {
    if status == OK { Ok(()) } else { bail!("NVAPI {what}: status {status}") }
}

/// The NVAPI entry points, from the driver's own `nvapi64.dll`.
struct Nvapi {
    query: QueryInterface,
}

impl Nvapi {
    fn load() -> Result<Self> {
        let name = wide("nvapi64.dll");
        // SAFETY: a NUL-terminated wide name; a missing DLL gives null.
        let module = unsafe { LoadLibraryW(name.as_ptr()) };
        if module.is_null() {
            bail!("nvapi64.dll not found (no NVIDIA driver)");
        }
        // SAFETY: a loaded module and a NUL-terminated name.
        let query = unsafe { GetProcAddress(module, c"nvapi_QueryInterface".as_ptr().cast()) };
        if query.is_null() {
            bail!("nvapi_QueryInterface not found");
        }
        // SAFETY: nvapi_QueryInterface takes one u32 and returns a pointer (cdecl).
        Ok(Nvapi { query: unsafe { std::mem::transmute::<*mut c_void, QueryInterface>(query) } })
    }

    /// The function `id` (or `fallback`, the older id of the same call) as a `F`.
    fn function<F: Copy>(&self, id: u32, fallback: u32) -> Result<F> {
        // SAFETY: QueryInterface only looks the id up.
        let mut found = unsafe { (self.query)(id) };
        if found.is_null() && fallback != 0 {
            // SAFETY: as above.
            found = unsafe { (self.query)(fallback) };
        }
        if found.is_null() {
            bail!("NVAPI function 0x{id:08X} not found");
        }
        assert_eq!(size_of::<F>(), size_of::<*mut c_void>());
        // SAFETY: `F` is the function pointer type NVAPI documents for that id.
        Ok(unsafe { std::mem::transmute_copy(&found) })
    }

    fn open(&self) -> Result<Session> {
        // SAFETY: the calls' documented signatures; the session pointer is ours.
        unsafe {
            check(self.function::<NoArgs>(0x0150_E828, 0)?(), "Initialize")?;
            let mut session: Session = std::ptr::null_mut();
            check(self.function::<CreateSession>(0x0694_D52E, 0)?(&mut session), "CreateSession")?;
            check(self.function::<OnSession>(0x375D_BD6B, 0)?(session), "LoadSettings")?;
            Ok(session)
        }
    }

    fn close(&self, session: Session) {
        if let Ok(destroy) = self.function::<OnSession>(0xDAD9_CFF8, 0) {
            // SAFETY: a session `open` made.
            unsafe { destroy(session) };
        }
    }

    /// The profile that holds the executable, if one does.
    fn profile_of(&self, session: Session) -> Result<Option<Handle>> {
        let name = wide(EXE);
        let mut profile: Handle = std::ptr::null_mut();
        let mut info = zeroed::<Application>();
        info.version = APPLICATION_VER;
        // SAFETY: an open session, a NUL-terminated name, out pointers ours.
        let status =
            unsafe { self.function::<FindApp>(0xEEE5_66B2, 0)?(session, name.as_ptr(), &mut profile, &mut *info) };
        match status {
            OK => Ok(Some(profile)),
            EXECUTABLE_NOT_FOUND => Ok(None),
            other => check(other, "FindApplicationByName").map(|()| None),
        }
    }
}

fn dword_setting(id: u32, value: u32) -> Box<Setting> {
    let mut setting = zeroed::<Setting>();
    setting.version = SETTING_VER;
    setting.id = id;
    setting.kind = 0; // DWORD
    setting.current_value[..4].copy_from_slice(&value.to_le_bytes());
    setting
}

/// Whether Smooth Motion is on for `victory.exe`: `None` when no profile holds it or the setting is
/// not set there (the driver's default, off).
fn get() -> Result<Option<bool>> {
    let nvapi = Nvapi::load()?;
    let session = nvapi.open()?;
    let result = (|| {
        let Some(profile) = nvapi.profile_of(session)? else { return Ok(None) };
        let mut setting = dword_setting(SM_ENABLE, 0);
        let mut extra = 0u32;
        // SAFETY: an open session, a profile of it, out pointers ours.
        let status = unsafe {
            nvapi.function::<GetSetting>(0xEA99_498D, 0x73BF_8338)?(session, profile, SM_ENABLE, &mut *setting, &mut extra)
        };
        if status == SETTING_NOT_FOUND {
            return Ok(None);
        }
        check(status, "GetSetting")?;
        let value = u32::from_le_bytes(setting.current_value[..4].try_into().expect("4 bytes"));
        Ok(Some(value != 0))
    })();
    nvapi.close(session);
    result
}

/// Turns Smooth Motion on or off for `victory.exe`, making its profile first if none holds it.
fn set(on: bool) -> Result<String> {
    let nvapi = Nvapi::load()?;
    let session = nvapi.open()?;
    let result = (|| {
        let mut made = String::new();
        let profile = match nvapi.profile_of(session)? {
            Some(profile) => profile,
            None => {
                let name = wide(PROFILE);
                let mut profile: Handle = std::ptr::null_mut();
                // SAFETY: an open session, a NUL-terminated name, an out pointer ours.
                let found =
                    unsafe { nvapi.function::<FindProfile>(0x7E4A_9A0B, 0)?(session, name.as_ptr(), &mut profile) };
                if found == PROFILE_NOT_FOUND {
                    let mut info = zeroed::<Profile>();
                    info.version = PROFILE_VER;
                    put(&mut info.name, PROFILE);
                    info.gpu_support = 1; // GeForce
                    // SAFETY: as above, with a filled profile description.
                    check(
                        unsafe { nvapi.function::<CreateProfile>(0xCC17_6068, 0)?(session, &mut *info, &mut profile) },
                        "CreateProfile",
                    )?;
                    made.push_str(&format!("driver profile \"{PROFILE}\" made; "));
                } else {
                    check(found, "FindProfileByName")?;
                }
                let mut app = zeroed::<Application>();
                app.version = APPLICATION_VER;
                put(&mut app.app_name, EXE);
                // SAFETY: an open session, a profile of it, a filled application description.
                check(
                    unsafe { nvapi.function::<CreateApp>(0x4347_A9DE, 0)?(session, profile, &mut *app) },
                    "CreateApplication",
                )?;
                made.push_str("victory.exe added to it; ");
                profile
            }
        };
        let write = nvapi.function::<SetSetting>(0x8A2C_F5F5, 0x577D_D202)?;
        let mut enable = dword_setting(SM_ENABLE, u32::from(on));
        // SAFETY: an open session, a profile of it, a filled setting.
        check(unsafe { write(session, profile, &mut *enable, 0, 0) }, "SetSetting(enable)")?;
        if on {
            let mut apis = dword_setting(SM_APIS, APIS_DX12);
            // SAFETY: as above.
            check(unsafe { write(session, profile, &mut *apis, 0, 0) }, "SetSetting(APIs)")?;
        }
        // SAFETY: an open session.
        check(unsafe { nvapi.function::<OnSession>(0xFCBC_7E14, 0)?(session) }, "SaveSettings")?;
        Ok(format!("{made}Smooth Motion {} for victory.exe", if on { "on" } else { "off" }))
    })();
    nvapi.close(session);
    result
}

/// `victory smooth-motion <what>`: `status`, `on` or `off`. Exit code 2 (an error) when the
/// driver's API is not there or refuses.
pub fn run(what: &str) -> Result<()> {
    match what {
        "status" => {
            let state = match get()? {
                Some(true) => "on",
                Some(false) => "off",
                None => "not set (the driver's default: off)",
            };
            println!("Smooth Motion for victory.exe: {state}");
        }
        "on" | "off" => println!("{}", set(what == "on")?),
        other => return Err(anyhow!("smooth-motion takes status, on or off, not {other:?}")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sizes NVAPI's structure versions encode (NVDRS_PROFILE_VER1, NVDRS_APPLICATION_VER_V4,
    /// NVDRS_SETTING_VER1).
    #[test]
    fn nvapi_structures_have_the_drivers_sizes() {
        assert_eq!(size_of::<Profile>(), 4116);
        assert_eq!(size_of::<Application>(), 20492);
        assert_eq!(size_of::<Setting>(), 12320);
    }
}
