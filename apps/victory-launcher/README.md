# VIctoryUnified graphical launcher

Build on Windows with `cargo build --workspace --locked --release`. Keep the resulting `VIctoryUnified.exe` and `victory.exe` together. The launcher copies the runtime and setup scripts to `%LOCALAPPDATA%\VIctoryUnified\v2.0` when it opens. Use `--verify` for a runtime CLI smoke test, `--page play|graphics|performance|tools` to open a specific page, or `--play` to launch with saved settings.

The Tools tab accepts a folder or ZIP of Story Mode slots and backs up existing saves before replacing slots. The game should be closed before importing. See [the build instructions](../../BUILD.md) for runtime dependencies.
