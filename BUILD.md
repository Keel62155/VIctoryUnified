# Build VIctoryUnified v2.0

## Prerequisites

- Windows 10 or 11 x64, Git, rustup, Visual Studio Build Tools with MSVC and a Windows SDK.
- The Rust version and components in `rust-toolchain.toml`. Rustup selects them automatically inside the source directory.
- Access to crates.io on the first build. Cargo resolves against the included lockfiles.

## Build and verify from a fresh extraction

```powershell
cargo build --workspace --locked --release
cargo test --workspace --locked --release
cargo clippy --workspace --all-targets --locked --release
cargo clippy --workspace --all-targets --all-features --locked --release
cargo xtask policy
$p = Start-Process .\target\release\VIctoryUnified.exe -ArgumentList '--verify' -Wait -PassThru
if ($p.ExitCode -ne 0) { throw "Launcher verification failed ($($p.ExitCode))" }
```

The workspace builds both executables into `target\release`. The launcher copies the locally built runtime and the setup scripts to `%LOCALAPPDATA%\VIctoryUnified\v2.0`. The verification command checks the installed runtime's CLI without loading a title. Clippy reports warnings, including unfinished GPU hooks preserved in the source. Some existing integration tests use optional title fixtures or graphics capabilities and skip those checks when the inputs are unavailable.

## Run

You need your own complete game dump, an AVX2-capable CPU, and a D3D12 GPU with shader model 6.6 and mesh shader support. Start `target\release\VIctoryUnified.exe` and pick the game folder. For the batch fallback use `START.bat [game-folder]` or set `VICTORY_GAME_DIR`. Use `START.bat warm` to warm the shader cache with six workers. With a custom Cargo target directory, set `VICTORY_RUNNER` to the built runtime before using the fallback.

- On NVIDIA in Series S mode, supply a compatible AMD PC FidelityFX upscaler DLL. Put it under `fsr\amd_fidelityfx_upscaler_dx12.dll` in the versioned local data directory or set `VICTORY_FSR_DLL` to its absolute path.
- Optional Microsoft Agility SDK, OptiScaler and NVIDIA Streamline dependencies are set up on first use by the scripts under `tools\`. OptiScaler's DLSS option uses a locally installed NVIDIA upscaling DLL. Streamline fetches the NVIDIA release directly and verifies its hash and signatures. These dependencies are unnecessary for compilation.
- The five recent experimental engine switches each require a value of `1`. See [PROGRESS.md](PROGRESS.md) for combinations and scope.
