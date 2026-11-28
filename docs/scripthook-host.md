# ScriptHook VI host adapter for VIctory

The Rust adapter is reusable across Windows x64 VIctory forks. It uses the
published hosted-start ABI and depends only on the Rust standard library.
A compatible runtime must call the adapter before handing control to the
mapped title. This kit contains the adapter and integration instructions.

## Add the adapter to a fork

1. Put `scripthook_host.rs` beside the runtime's loader entry point. Include
   it with `#[cfg(windows)] mod scripthook_host;`.
2. In the execute path, once the title image is mapped and its imports are
   bound, call the adapter before entering title code. Pass the actual mapped
   address, PE image size, PE timestamp, and game directory. In VIctory-based
   trees, the call typically looks like this.

   ```rust
   crate::scripthook_host::start(
       title_dir,
       image.base(),
       image.info().size_of_image as u64,
       image.info().timestamp,
   );
   ```

3. Build the fork normally. A fork that already implements the hosted
   `victory_on_title_mapped` ABI can use its existing loader integration. A
   fork that exposes no hosted callback needs the loader change. Loading the
   core as a generic DLL does not supply the mapped title's address or PE data.

## Set up the local opt-in

Build the official ScriptHook VI source at
`57b1946d1a72ef055766ac76f2145151ee53feb6` with Visual Studio 2022 and CMake.
Run these commands from the root of that checkout.

```powershell
cmake -S . -B <build-dir> -G "Visual Studio 17 2022" -A x64
cmake --build <build-dir> --config Release --target shvi_core trainer --parallel 6
```

Stage the files in a directory outside the game package.

| Destination under the local runtime folder | Input |
| --- | --- |
| `shvi/shvi_core.dll` | `<build-dir>/Release/shvi_core.dll` |
| `shvi/plugins/trainer.asi` | `<build-dir>/shvi/plugins/trainer.asi` |
| `shvi/natives/1794606036.json` | `nativedb/db/1794606036/native_table_full.json` |

Set `VICTORY_SCRIPTHOOK_DIR` to that folder and `VICTORY_SCRIPTHOOK=1` for an
offline Story Mode launch. The default directory is `../scripthook` relative
to the directory containing the runtime executable. With the switch unset or
set to `0`, the adapter returns before looking for the core. In Story Mode,
the upstream trainer uses F4. Its offline guard must confirm the game mode
before scripts can run.

The currently supported Xbox Series image has PE timestamp `1794606036`.
The core logs translated native bindings and offline guard transitions in
`shvi/shvi.log`. A successful hosted startup can still be followed by an
initialization failure. Inspect that log before claiming the trainer works
with a given fork or native patch set.

## Redistribution

This adapter is supplied under GPL-3.0-or-later. ScriptHook VI has no
redistribution license at the pinned revision. Each user needs to build it
locally. NativeDB is licensed CC BY 4.0 and requires attribution if shared.
