//! The DirectX 12 Agility SDK, loaded at run time when it is beside the executable.
//!
//! Windows 10's own Direct3D 12 stops at shader model 6.5 and refuses the heap-indexed root
//! signatures every translated pipeline uses (Marvisak/VIctory #175, #463, issue #753). The
//! Agility SDK's `D3D12Core.dll` (Microsoft.Direct3D.D3D12 1.620.2, put in `D3D12\` beside the
//! executable by `tools\setup-agility.ps1`) gives shader model 6.6 there, and the newer runtime
//! elsewhere.
//!
//! Loaded through `ID3D12SDKConfiguration1::CreateDeviceFactory` and made the process's runtime
//! with `ApplyToGlobalState` (the way of Marvisak/VIctory #969), not through the executable's
//! `D3D12SDKVersion` / `D3D12SDKPath` exports: with those exports and no DLL, every device is
//! refused (`D3D12_ERROR_INVALID_REDIST`, 0x887E0003, seen here; as oliking800's
//! victory-agility-patcher warns). Here a missing `D3D12\`, a refused SDK or any other failure
//! leaves Windows' own runtime, as before. `VICTORY_AGILITY=0`: not loaded.

#![cfg(windows)]

use std::sync::OnceLock;

/// The SDK version of the `D3D12Core.dll` the setup puts there (1.620.2).
const SDK_VERSION: u32 = 620;

/// Loads the SDK if it is there, once, before the first device: the SDK version in use, or
/// `None` for Windows' own runtime.
pub fn load() -> Option<u32> {
    static LOADED: OnceLock<Option<u32>> = OnceLock::new();
    *LOADED.get_or_init(|| {
        if std::env::var_os("VICTORY_AGILITY").is_some_and(|v| v == "0") {
            tracing::info!("Direct3D 12 Agility SDK not loaded (VICTORY_AGILITY=0): Windows' own runtime");
            return None;
        }
        let dir = std::env::current_exe().ok()?.parent()?.join("D3D12");
        if !dir.join("D3D12Core.dll").is_file() {
            tracing::info!("no D3D12\\D3D12Core.dll beside the executable: Windows' own Direct3D 12 runtime");
            return None;
        }
        match apply(&dir) {
            Ok(()) => {
                tracing::info!(
                    version = SDK_VERSION,
                    dir = %dir.display(),
                    "Direct3D 12 Agility SDK loaded (VICTORY_AGILITY=0: Windows' own runtime)"
                );
                Some(SDK_VERSION)
            }
            Err(e) => {
                tracing::warn!(error = %e, dir = %dir.display(), "Direct3D 12 Agility SDK not loaded: Windows' own runtime");
                None
            }
        }
    })
}

fn apply(dir: &std::path::Path) -> windows::core::Result<()> {
    use windows::Win32::Graphics::Direct3D12::{
        CLSID_D3D12SDKConfiguration, D3D12GetInterface, ID3D12DeviceFactory, ID3D12SDKConfiguration1,
    };
    // The SDK path with its trailing separator, as a C string.
    let mut path = dir.as_os_str().to_string_lossy().into_owned();
    path.push('\\');
    let path = std::ffi::CString::new(path).map_err(|_| windows::core::Error::from(windows::Win32::Foundation::E_INVALIDARG))?;
    // SAFETY: the documented calls; `path` outlives them.
    unsafe {
        let mut config: Option<ID3D12SDKConfiguration1> = None;
        D3D12GetInterface(&CLSID_D3D12SDKConfiguration, &mut config)?;
        let config = config.ok_or_else(|| windows::core::Error::from(windows::Win32::Foundation::E_NOINTERFACE))?;
        let factory: ID3D12DeviceFactory = config.CreateDeviceFactory(SDK_VERSION, windows::core::PCSTR(path.as_ptr().cast()))?;
        factory.ApplyToGlobalState()
    }
}
