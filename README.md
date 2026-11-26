# cool_fox for 8 GB NVIDIA cards

A fork of **cool_fox** (codec_d's fork of [VIctory](https://github.com/Marvisak/VIctory), a
compatibility layer that runs an Xbox Series title on Windows from files the user supplies),
with fixes for **NVIDIA RTX cards with 8 GB of video memory**: correct lighting and colour,
every 64-lane GPU program translated exactly, and settings measured on an RTX 4060 8 GB.

**This repository contains no game files, code, assets, keys or download links.** You need
your own complete, lawfully obtained copy of the game. Not affiliated with or endorsed by the
game's developer or publisher.

## Where it stands

Tested on one PC: Intel i7-14700F, RTX 4060 8 GB, 32 GB RAM, Windows 11, Resizable BAR on,
Series S mode, the prologue.

| | |
|---|---|
| Prison cell, the prologue's first rooms | about 18-21 fps (48-56 ms a frame) |
| Corridors | about 20-22 fps |
| Common room (busier) | about 15-18 fps |
| Picture | correct lighting and colour gradients; no banding, blotches or black boxes |

What was fixed here, and where each fix came from, with its measured effect:
[PROGRESS.md](PROGRESS.md).

- **Dark rooms, black silhouettes, rings around lamps:** `VICTORY_LEVELS_ON_DEMAND` turned off
  (found by bisecting cool_fox's switch set; it costs no speed).
- **Colour banding, blotchy walls and skin, black boxes on faces:** the title's 64-lane
  (wave64) compute programs translated exactly on NVIDIA's 32-lane waves, two console lanes per
  host lane (`VICTORY_WAVE64_PAIRED=1`, ported from kelvaroq). Before, cool_fox ran about 25 of
  them in an approximate form whose results were wrong, and dropped four.
- **Black screen in Series S mode:** the title's FSR upscale goes to AMD's PC FSR DLL
  (`VICTORY_FSR_DLL`, ported from Neptune), plus a hang fix in that path.
- **Speed:** game buffers in the card's memory (`VICTORY_CARD_BUFFERS_MB=1536`, about 2x),
  Resizable BAR (`VICTORY_WINDOWS_BAR_MB=1024`), 32-bit lighting targets
  (`VICTORY_E5_TARGETS=r11g11b10`, -15 %), a whole-program-optimized build (`fast` profile, -7 %).

## How to run it

You need Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with shader model 6.6 and mesh
shaders, Rust (the toolchain is pinned in `rust-toolchain.toml`), the MSVC build tools with a
Windows SDK, and your own dump of the title.

```powershell
# The release build:
cargo build --release --locked -p victory
# Or, faster, for your own processor only (about 7 % faster here):
$env:RUSTFLAGS = '-Ctarget-cpu=native'
cargo build --profile fast --locked -p victory
```

Then run `scripts\play-nvidia-8gb.cmd "<your game folder>"` (the folder with
`game_scarlett_master.exe`). It translates every shader once (a few minutes), then starts the
game in Series S mode with the settings above. The first session stutters while the graphics
driver builds its pipelines; later ones do not.

Series S mode needs AMD's PC FSR upscaler DLL, `amd_fidelityfx_upscaler_dx12.dll`, from a PC
game you own that ships it (it is not included here). Put its path in `VICTORY_FSR_DLL`, or the
DLL in a folder named `fsr` next to the launcher. Without it the picture stays black on NVIDIA.

Optional: `VICTORY_ICON=<file.ico>` sets the game window's icon from a file of your own.

## What needs to be done

Help is welcome. In rough order of importance:

1. **Speed: the submitting thread.** About 40 ms of each frame is CPU preparation on one
   thread: about 20 ms copying 40 MB of shader results back to title memory, 8 ms uploading
   56 MB. More translation threads made it slower (6 was best on a 20-core Intel CPU; 12
   and 16 were worse than the default 8), and keeping more threads on P-cores did too. Ideas: share the copy-back across threads, skip
   copy-backs the title never reads (Dolberthi's "smart copy-back", carefully: stale data
   breaks game logic), profile-guided optimization.
2. **Speed: the card.** The card is 87-92 % busy at 720p. `VICTORY_BVH_MISS=1` (ray walks end
   at once while the structure builder is off) saved about 2 ms in a first test; its effect on
   the picture is not checked yet. `VICTORY_RENDER_SCALE` (the title's own render scale, in
   memory) is ported but not tuned.
3. **Stretched inmate clothing ("light slabs") and an orange shard near the guard.** Skinned
   meshes are handed back before the card has finished them. `VICTORY_RELAY=1` and
   `VICTORY_CARD_STEPS=0` remove them but cost 27 and 19 ms a frame; a targeted fix (wait only
   for the skinning outputs) is needed.
4. **Rainbow / pixelated skin** on some characters: a texture refresh that does not wait
   (`VICTORY_TEXTURE_NO_WAIT=0` fixes it at about 4 ms), and probably the 16-bit "halves" fix
   (fxceless/victory-fork #927), not ported yet.
5. **Haze and silhouettes in the common room by its sunlit windows:** without ray tracing the
   light probes keep the sky's light indoors. The ambient stand-in limits it; a real fix needs
   indirect light that follows the room.
6. **The 30 fps cap:** the title's own frame interval; the "willy" cool_fox build has an unlock
   (`VICTORY_FRAME_INTERVAL_MS`), not ported yet.
7. **Ray tracing** on 8 GB cards: very slow, and the title's watchdog ends slow frames
   (`VICTORY_NO_WATCHDOG=1` patches it out in memory).

## Not included, on purpose

- The instruction emulation cool_fox ported from XCL (`console_insn.rs`), which is
  GPL-2.0-only and cannot be combined with this GPL-3.0 program. VIctory's own emulation of the
  same instructions (`clzero`, `monitorx`/`mwaitx`, `rdpru`, SSE4a) takes its place.
- Any game file, artwork, DLL or shader of the title, and any icon.

## Credits

- [VIctory](https://github.com/Marvisak/VIctory) by Marvisak and contributors: the loader,
  runtime services, Direct3D 12 back end and shader recompiler.
- **cool_fox** by codec_d, and the forks it is built on: avatarmanz/VIctory-7900XTX,
  fxceless/victory-fork, DexNom, Belobug, FQY55kai, myokex (see `docs/README-cool_fox.md`).
- **kelvaroq** (stellarr-studio): the paired-lane wave64 translation.
- **Neptune** (zm0kie and contributors): the FidelityFX redirect.
- **VIctory-NVIDIA** (trustno1-coder): pipeline wait-once, `VICTORY_BVH_MISS`, the ambient cap,
  and the in-memory title patches it took from victory-perf / ScarlettPort.
- The **"willy" cool_fox build**: the R11G11B10 lighting targets.

## License

GPL-3.0-or-later, as VIctory. See [LICENSE](LICENSE); the one file that is not covered is named
in [docs/asset-policy.md](docs/asset-policy.md).
