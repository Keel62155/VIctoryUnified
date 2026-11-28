# VIctoryUnified

**An enhanced, performance-focused fork of VIctory for Windows, combining renderer optimizations, improved NVIDIA compatibility, native DLSS Frame Generation and features from multiple community projects.**

VIctoryUnified is an experimental Xbox Series X|S compatibility layer designed to run Grand Theft Auto VI on Windows using game files supplied by the user.

The project focuses on improving real gameplay performance, reducing memory consumption, minimizing open-world stuttering and providing an accessible experience through a dedicated graphical launcher and in-game settings.

**[Download Latest Release](https://github.com/Keel62155/VIctoryUnified/releases/latest) • [Report an Issue](https://github.com/Keel62155/VIctoryUnified/issues) • [Build Instructions](BUILD.md) • [VctoryTrainer](https://github.com/Keel62155/VctoryTrainer)**

---

## Features

### Graphics & Rendering

- Native NVIDIA DLSS upscaling.
- Native NVIDIA DLSS Frame Generation through Streamline.
- Multiple DLSS quality modes and model presets.
- AMD FidelityFX Super Resolution (FSR).
- Xbox Series X and Series S rendering profiles.
- Adjustable rendering resolution.
- Configurable lighting, reflections, fog and water effects.
- Optional ray tracing.
- Configurable VSync and frame-rate limits.
- Fullscreen support.

### Performance

- Optimizations targeting NVIDIA GPUs, particularly cards with 8GB of VRAM.
- Improved CPU and GPU synchronization.
- Asynchronous command submission.
- Multithreaded graphics command translation.
- Improved descriptor synchronization.
- Optimized memory-transfer operations.
- Texture streaming and residency improvements.
- Shader translation and pipeline caching.
- Configurable CPU worker pools.
- Resizable BAR support.
- Experimental CPU scheduling optimizations.
- Shader cache warming using six workers.

### Graphical Launcher

VIctoryUnified includes a dedicated graphical launcher designed to simplify installation, configuration and gameplay.

- Automatic game directory validation.
- Max Performance, Balanced, Quality and Custom profiles.
- Graphics and performance configuration.
- Persistent user settings.
- Dependency management.
- Shader cache management.
- Save importing and management.
- Access to runtime logs.
- Advanced launch switches.

### In-Game Settings

Press **F1** to open the in-game settings menu.

Configure DLSS, Frame Generation, rendering profiles, graphical effects, memory management and advanced performance options without manually editing launch scripts.

Some settings apply immediately, while others require restarting the game.

### Performance Overlay

Press **F3** to display detailed performance statistics, including:

- Real FPS.
- Frame-generated FPS.
- Frame times.
- CPU submission performance.
- GPU processing and waiting times.
- Draw calls.
- Memory transfers.
- Dedicated and shared GPU memory usage.
- System RAM usage.

### Single-Player Modding

VIctoryUnified supports optional ScriptHook VI integration through VctoryTrainer.

When installed and enabled, press **F4** to access the in-game trainer.

- Single-player Story Mode support.
- Native game function invocation.
- In-game trainer interface.
- No external DLL injector required.
- Original game files remain untouched.

Learn more at the [VctoryTrainer repository](https://github.com/Keel62155/VctoryTrainer).

---

## System Requirements

VIctoryUnified is still under active development. Performance and compatibility may vary considerably between hardware configurations.

| Component | Requirement |
|---|---|
| Operating System | Windows 10/11 x64 |
| Processor | x86-64 CPU with AVX2 support |
| Graphics API | DirectX 12 |
| GPU Features | Shader Model 6.6 and mesh shader support |
| Graphics Memory | 8GB or more recommended |
| System Memory | 32GB or more recommended |
| Storage | SSD strongly recommended |
| Resizable BAR | Recommended |
| Game Files | Your own complete, compatible Xbox Series game files |

### Primary Development Hardware

VIctoryUnified is primarily developed and tested on the following system:

| Component | Specification |
|---|---|
| CPU | Intel Core i7-14700F |
| GPU | NVIDIA GeForce RTX 4060 8GB |
| RAM | 32GB |
| Operating System | Windows 11 |
| Resizable BAR | Enabled |

Higher-end hardware may provide better performance, but the project aims to improve accessibility for users with mid-range systems.

---

## Installation

### Precompiled Release

1. Download the latest Windows x64 release from the [Releases page](https://github.com/Keel62155/VIctoryUnified/releases).
2. Extract the ZIP into a folder of your choice.
3. Keep `VIctoryUnified.exe` and `victory.exe` together.
4. Open `VIctoryUnified.exe`.
5. Select your compatible game directory.
6. Choose your preferred graphics and performance settings.
7. Press **PLAY**.

The first launch may take longer because shaders and graphics pipelines need to be prepared.

Do not interrupt shader preparation unnecessarily.

### Recommended Starting Settings

For NVIDIA GPUs with 8GB of VRAM:

| Setting | Recommended |
|---|---|
| Performance Preset | Max Performance |
| Console Profile | Xbox Series S |
| Upscaling | NVIDIA DLSS or FSR |
| DLSS Quality | Performance |
| Frame Generation | Off initially |
| Ray Tracing | Off |
| Resizable BAR | Enabled |
| Shader Cache | Fully prepared |

Once the game is running reliably, experiment with native 2× DLSS Frame Generation on supported hardware.

Frame Generation improves displayed smoothness but does not increase the underlying game simulation rate.

---

## Keyboard Shortcuts

| Key | Function |
|---|---|
| F1 | In-game settings menu |
| F3 | Performance overlay |
| F4 | Optional VctoryTrainer menu |

Additional input configuration may be available through the launcher and `bindings.ini`.

---

## Building From Source

### Requirements

- Rust using the pinned repository toolchain.
- Microsoft Visual Studio C++ Build Tools.
- Windows SDK.
- Git.

Clone the repository:

```bash
git clone https://github.com/Keel62155/VIctoryUnified.git
cd VIctoryUnified
```

Build the runtime:

```bash
cargo build --release --locked -p victory
```

The compiled runtime will be available at:

```text
target/release/victory.exe
```

For graphical launcher compilation, dependency setup and release packaging, refer to [BUILD.md](BUILD.md).

---

## Advanced Configuration

VIctoryUnified supports advanced configuration through `VICTORY_*` environment variables.

Example performance configuration:

```ini
VICTORY_EARLY_DEPTH=1
VICTORY_SYNC_FINE=1
VICTORY_SYNC_IN_PLACE=1
VICTORY_TEXTURE_NO_WAIT=1
VICTORY_TEXTURE_POOL=1
VICTORY_RELAY=2
VICTORY_RELAY_COPY=1
```

Advanced settings may affect performance, rendering correctness and stability.

Not every configuration is appropriate for every hardware combination. Always test experimental settings before using them permanently.

See the [documentation](docs/) and [example configuration](victory.example.toml) for additional information.

---

## Current Development Goals

The primary goal of VIctoryUnified is to improve actual gameplay performance rather than simply achieve higher FPS in menus or stationary scenes.

- [x] Graphical Windows launcher.
- [x] NVIDIA DLSS integration.
- [x] Native DLSS Frame Generation integration.
- [x] In-game settings menu.
- [x] Detailed performance overlay.
- [x] Optional ScriptHook VI trainer integration.
- [ ] Sustained 20–25 real FPS during demanding free roam.
- [ ] Improved performance while driving through populated areas.
- [ ] Significantly reduced open-world stuttering.
- [ ] Further VRAM and system memory reductions.
- [ ] Improved CPU/GPU memory coherence.
- [ ] More efficient demand-driven memory synchronization.
- [ ] Improved graphical accuracy.
- [ ] Expanded hardware compatibility.

Development progress and technical findings are documented in [PROGRESS.md](PROGRESS.md).

---

## Known Issues

VIctoryUnified is experimental software and should not be considered a finished compatibility solution.

Current limitations may include:

- Significant FPS drops in demanding open-world locations.
- Texture streaming delays.
- Shader and pipeline compilation stutters.
- Occasional graphical artifacts.
- High system RAM and GPU memory consumption.
- Experimental settings causing instability or graphical corruption.
- Frame Generation behavior varying with hardware, window focus and presentation mode.
- Incomplete compatibility with some hardware configurations.

A high displayed FPS with Frame Generation enabled does not necessarily indicate high real gameplay performance.

---

## Troubleshooting

If you experience crashes, missing textures or severe performance problems:

- Try the default or Max Performance preset.
- Disable Frame Generation.
- Disable experimental rendering options.
- Use the Series S profile.
- Verify that your GPU drivers are up to date.
- Allow shader preparation to finish.
- Check available system RAM and VRAM.
- Close unnecessary background applications.
- Review the runtime logs.

When reporting an issue, please include:

- CPU and GPU.
- System RAM and VRAM.
- Windows version.
- GPU driver version.
- VIctoryUnified version.
- Console and rendering profile.
- Relevant launch settings.
- Runtime logs.
- A description of the problem and how to reproduce it.

Do not upload game files, personal saves or sensitive information.

[Report an issue here](https://github.com/Keel62155/VIctoryUnified/issues).

---

## Contributing

Contributions, performance research, bug reports and compatibility testing are welcome.

Areas where contributions are particularly appreciated:

- CPU and GPU performance.
- Xbox unified memory emulation.
- Texture streaming.
- Shader translation.
- DirectX 12 rendering.
- Memory synchronization.
- NVIDIA and AMD compatibility.
- Frame pacing and stuttering.
- Launcher improvements.
- Documentation.

Please ensure that contributions respect the project's licensing requirements and do not contain proprietary game code, restricted SDK files or copyrighted game assets.

Performance-related pull requests should include reproducible benchmark results wherever possible.

---

## Credits & Acknowledgements

VIctoryUnified builds upon the work of numerous developers and community projects. This project would not exist without their contributions.

### Project Development

**[Keel62155](https://github.com/Keel62155)**

VIctoryUnified development, integration, hardware testing, packaging and maintenance.

### Upstream Projects & Contributors

- **[Marvisak — VIctory](https://github.com/Marvisak/VIctory)** — Original compatibility layer, runtime services, graphics backend and shader recompilation.
- **codec_d — cool_fox** — Renderer and performance optimization work.
- **[trustno1-coder — VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA)** — NVIDIA renderer improvements, DLSS, Frame Generation and in-game settings.
- **[avatarmanz — VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX)** — AMD renderer and memory-management optimizations.
- **[fxceless — victory-fork](https://github.com/fxceless/victory-fork)** — Renderer, synchronization and performance improvements.
- **[DexNom — VIctory](https://github.com/DexNom/VIctory)** — Rendering fixes, ray tracing and sparse texture research.
- **[Biskwit — VIctory-reloaded](https://github.com/Biskwit/VIctory-reloaded)** — CPU scheduling and command submission research.
- **[shuffl3r — victory-nvidia](https://github.com/shuffl3r/victory-nvidia)** — NVIDIA Streamline integration research.
- **Belobug and romainhedouin** — Ambient lighting and pipeline worker improvements.
- **FQY55kai — ZJOU-VICTORY** — Upstream compatibility and optimization work.
- **myokex, Spexivus, joroyt, tightfall and Joshua Odom** — Memory guards and title memory research.
- **Geckomacabre and cxy0072019-sketch** — Texture readback improvements.

### Modding & Trainer Development

- **[modpapi — ScriptHook VI](https://github.com/modpapi/scripthook-vi)** — Original script hook, native function research and trainer implementation.
- **[NitricAcidDev — VIctory-Omega-Fork](https://github.com/NitricAcidDev/VIctory-Omega-Fork)** — ScriptHook VI host integration reference.
- **Anh** — Special thanks for helping discover ScriptHook VI and pointing the project in the right direction.

### Additional Acknowledgements

Thank you to NVIDIA, AMD and the open-source graphics development community for their publicly available technologies, documentation and development tools.

Thank you to everyone who has tested VIctoryUnified, reported bugs, shared performance research or contributed to the wider VIctory community.

Development has been assisted by Claude Code and OpenAI Codex.

See [THIRD_PARTY.md](THIRD_PARTY.md) for additional attribution and licensing information.

---

## License

VIctoryUnified is distributed under the **GNU General Public License v3.0 or later**, consistent with the upstream VIctory project.

See [LICENSE](LICENSE) for details.

Third-party components remain subject to their respective licenses and distribution terms. NVIDIA Streamline licensing information is available in [LICENSE-Streamline.txt](LICENSE-Streamline.txt).

Proprietary runtime dependencies and separately licensed modding components must be obtained independently where required.

---

## Disclaimer

VIctoryUnified is an independent, experimental compatibility project.

It is not affiliated with, endorsed by or sponsored by Rockstar Games, Take-Two Interactive, Microsoft, NVIDIA or AMD.

This repository does not distribute game files, game executables, copyrighted game assets or restricted proprietary SDKs.

Users must provide their own legally obtained, compatible game files.

Optional trainer functionality is intended exclusively for offline, single-player gameplay.

Use this software at your own risk and always back up your saves.

---

**VIctoryUnified — Bringing the best of VIctory together.**

If you enjoy the project, consider leaving a star on GitHub!
