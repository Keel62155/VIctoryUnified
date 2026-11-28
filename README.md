# VIctoryUnified v2.0

VIctoryUnified is a Windows x64 compatibility layer based on VIctory and cool_fox. It runs the Xbox Series title from a game dump you supply. This source release contains the runtime, graphical launcher and build and setup scripts.

## Build and start

Install Rust via rustup, Git, and Visual Studio Build Tools with the **Desktop development with C++** workload and a Windows SDK. From the extracted source directory:

```powershell
cargo build --workspace --locked --release
.\target\release\VIctoryUnified.exe
```

Choose the folder containing `game_scarlett_master.exe` in the launcher. Keep the locally built `victory.exe` beside the launcher. Use `.\START.bat` for the command-line fallback or `.\START.bat warm` to translate shaders with exactly six workers. Settings, saves, logs and shader caches default to a versioned directory under `%LOCALAPPDATA%\VIctoryUnified\v2.0`.

See [BUILD.md](BUILD.md) for verification commands, hardware and runtime dependencies. See [THIRD_PARTY.md](THIRD_PARTY.md) for credits and licenses.

## Features and verification

| Feature | Status |
| --- | --- |
| Graphical launcher | Play, Graphics, Performance and Tools pages, with Balanced, Max performance and Quality presets |
| Shader cache warming | Six workers in the launcher and batch fallback |
| Native DLSS Frame Generation | Experimental. Streamline handles generated presents while FSR supplies upscaling. Frame generation is off by default |
| DLSS upscaling and optional FSR Frame Generation through OptiScaler | Optional runtime setup from a pinned release |
| NVIDIA Smooth Motion and ray tracing | Experimental options |
| CPU placement, split submission, offline preparation and Reflex sleep | Preserved as opt-in engine switches |

A 60 Hz frame interval applies to menus and videos where supported. Generated presents are separate from rendered gameplay frames. See [PROGRESS.md](PROGRESS.md) for historical measurements and the scope of source verification.

Licensed under GPL-3.0-or-later. See [LICENSE](LICENSE) and the [asset policy](docs/asset-policy.md).
