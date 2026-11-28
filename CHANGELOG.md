# Changelog

## v2.0

- Added the VIctoryUnified graphical launcher with saved settings, graphics options, three presets, shader warming and save import.
- Added experimental native DLSS Frame Generation through NVIDIA Streamline, including D3D12 integration, frame inputs and Reflex support. Optional OptiScaler and driver Smooth Motion paths remain available.
- Preserved queue and descriptor preparation work, worker placement changes and fidelity/upscaling integration from the latest working tree. The five recent ports are off by default and can be enabled individually.
- Restored the GPL-compatible loader path used in v1.1 for the source release.
- Made the graphical launcher build alongside the runtime from source without a pre-existing payload. Kept the `START.bat` fallback and six-worker shader cache warm-up.
- Kept runtime downloads optional and version-pinned. The archive ships source, setup scripts, documentation and license notices.
