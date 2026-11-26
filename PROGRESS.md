# cool_fox on an RTX 4060 8 GB: progress

PC: Intel i7-14700F (8 P + 12 E cores, 28 threads), RTX 4060 8 GB, 32 GB RAM, Windows 11,
Resizable BAR on (since 2026-11-26). Base: codec_d's cool_fox (commit e12c29f).

Measured with `cf-bench\bench.ps1` (outside this repository): a fresh copy of one saved
checkpoint each run, A pressed only through the start screens, median frame time of the
window title's samples from 120 s on (the prison cell). Runs while the PC is in other use
vary by about +-10 %; important results were repeated.

## Rendering

| Change | Source | Effect |
|---|---|---|
| Series S upscale through AMD's PC FSR DLL (`VICTORY_FSR_DLL`) | Neptune (`fidelityfx.rs`, `external.rs`) | the title's own FSR shader does not translate on NVIDIA |
| Thin lists wake `caught_up()` waiters | Neptune | hang at the first FSR dispatch, watchdog 30 s later |
| `VICTORY_LEVELS_ON_DEMAND` off (launcher) | bisected here | dark rooms, black silhouettes, rings; no speed cost |
| Paired-lane wave64 translation (`VICTORY_WAVE64_PAIRED=1`) | kelvaroq 0052 (after XCL 3ecf4cf) | colour banding, blotches and black squares gone; every wave64 program translates |
| `VICTORY_AMBIENT_MOST`, NaN-free air | VIctory-NVIDIA | opt-in; did not remove the common room's slabs |
| `VICTORY_F16_NEAREST=1` (launcher) | cool_fox's own switch | console rounding; no measured effect on banding |

Open: stretched inmate-clothing meshes (light "slabs") and an orange shard near the guard,
tied to `VICTORY_RELAY=2` / `VICTORY_CARD_STEPS=2` (gone with the safe settings, which cost
+27 / +19 ms); rainbow skin (likely the 16-bit "halves" fix, fxceless #927, not ported yet).

## Performance

| Change | Source | Cell, ms/frame |
|---|---|---|
| base (codec_d's pareto set) | cool_fox | 107-112 |
| `VICTORY_CARD_BUFFERS_MB=1536` | measured here | 49-69 (about 2x) |
| `VICTORY_WINDOWS_BAR_MB=1024` (Resizable BAR) | cool_fox switch, VIctory-coolfox-nvidia README | 59.8 -> 48.8 |
| pipeline wait-once | VIctory-NVIDIA (fxceless #1181) | stalls only; no average change expected |
| `VICTORY_BVH_MISS=1` | VIctory-NVIDIA | 58.1 -> 56.4 (opt-in) |
| `VICTORY_THIN_THREADS` 6 / 12 / 16 | measured here | 50.2 / 63.1 / 71.5 (8: 60.2); 6 is the default of the launchers |
| `VICTORY_FAST_CORES=3` | measured here | 66.5 (worse) |
| `VICTORY_WINDOWS_BAR_MB=2048` | measured here | 59.4 (no gain over 1024) |
| Neptune's CPU switch set | Neptune play.ps1 | crashed |
| `fast` build profile (fat LTO, 1 codegen unit, `-Ctarget-cpu=native`) | measured here | 60.2 -> 55.9 |
| `VICTORY_E5_TARGETS=r11g11b10` (R9G9B9E5 targets as 32-bit R11G11B10_FLOAT) | "willy" cool_fox build / Neptune-VE | 60.2 -> 51.2, no banding |

Where the time goes (default settings, cell): about 40 ms a frame of CPU preparation, of
it about 20 ms copying 40 MB of shader results back to title memory and 8 ms uploading
56 MB; the card is 87-92 % busy. The submitting thread works 41-42 ms a frame.

Looked at and not taken: Dolberthi's persistent worker pool (cool_fox spawns threads only
for copies of 32 MB and more, where the spawn is about 1 % of the copy); his smart
copy-back (cool_fox's card-side compare already fetches only changed pieces; skipping the
rest risks stale data the title reads).
