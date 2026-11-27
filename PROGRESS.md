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
| `VICTORY_TEXBACK_ASYNC` (texture read-backs on their own thread) | victory-nvidia-megafork | cell 51.3 -> 50.2 (none there); **off by default**: at half scale (3 read-backs of 350 KB a frame) rooms went purple and blotchy, black boxes on hair |
| pipeline build panic completes the job | fxceless #1353 via victory-nvidia-megafork | no speed effect (recovery only) |
| `VICTORY_RENDER_SCALE=0.5` through the FSR DLL | victory-perf patches | 43.5, but the picture is black |
| `VICTORY_RENDER_SCALE=0.5 VICTORY_OWN_UPSCALER=1` | VIctory-NVIDIA / megafork launch set | cell 51.3 -> 45.2 (22 fps), picture right, softer; the card still 84-95 % busy |
| plus `VICTORY_BVH_MISS=1` | VIctory-NVIDIA | 50.3, card 74 % busy, hazier light: not taken |
| own upscaler at half scale, on the route | | rooms turn purple in the common room and the picture smears on camera turns: **not shipped** |
| `VICTORY_RENDER_SCALE=0.5` through the FSR DLL, FSR's maximum left alone (now the default of the patch) | measured here | picture right; card 69 % busy; 51-58 ms in the cell: **the CPU now sets the frame time** |
| compares and rehashes batched per copy (`VICTORY_CARD_BATCH`, tried) | measured here | 53.5 -> 54.2 ms, no gain: 400 compared ranges a frame are 400 different copies, and the rehash list's time is its uploads; reverted |

### 2026-11-27: the submitting thread, other forks, Agility SDK, 60 fps (VIctoryUnified v1.1)

Measured with the walk-free cell runs (half scale through FSR, 200 s, frame times from 160 s,
"steady") on one build; identical settings spread about +-5 % between runs, so one run says little
under that.

| Change | Source | Result | Default |
|---|---|---|---|
| `card_dirty` looks only at copies touched since it last ran (`VICTORY_TOUCHED_LIST`) | here | same results by construction (test); new build 24.1 steady over 3 runs, installed v1.0 exe 23.65 over 2 | on |
| write-watch counters: an asking costs 1.5-2 us per MB of range with the title running, 6.5 us if it resets a page; askings from several threads do not overlap; submissions ask about ~1 GB a frame (4 ms) | here (`ww` measure) | | logging |
| asking only the refreshed ranges (`VICTORY_ASK_EXACT`), a 2 MiB gap (`VICTORY_ASK_GAP_KB`) | here | 1,000 MB a frame asked either way; no gain | off |
| descriptor sync begun first (`VICTORY_HEAP_SYNC_FIRST`) | scarlet-fps | within the spread | off |
| `VICTORY_ASK_QUIET=30` | scarlet-fps (after Neptune) | 21.1 against 24.1: worse | off |
| `VICTORY_COPY_BESIDE` + `STORE_BESIDE` | scarlet-fps / 3070 | 23.1: no gain | off |
| `VICTORY_GPU_BACK_MAX_KB=1024` | scarlet-fps | 26.7, but the game behaved differently (the story moved on otherwise): what its code reads of shader results is no longer there | off |
| `VICTORY_PORT_CONSTANTS` | scarlet-fps | never applies here: the 174 MB heap is one section in rotated order (three views); foxhound reports it slower on a 12 GB card | off |
| decal heap defragmentation left out (`VICTORY_MEASURE_LEAVE_OUT=02384a60fda6f2d0`, measuring only) | | 23.7 against 24.1: the pass costs ~1 ms or less in our paired-lane translation (11.85 ms in scarlet-fps'); nothing to skip or optimise | never |
| DLSS through OptiScaler 0.9.5 (`START-DLSS.bat`) | OptiScaler | 23.2 against 24.1, picture right | optional |
| DirectX 12 Agility SDK 1.620.2 at run time (`agility.rs`) | Marvisak/VIctory #969 | exe exports without the DLL refuse every device (0x887E0003, measured): loaded through `ID3D12SDKConfiguration1` instead; loads with the folder, Windows' runtime without (both checked) | set up by the launcher |
| 60 fps frame interval (`VICTORY_FRAME_INTERVAL_MS=16.667`) | scarlet-fps | menus and videos at 60 | launcher on |
| opt-in, not measured yet: `VICTORY_SECTION_MB` (Vilocity), `VICTORY_SYNC_FINE`, `VICTORY_SYNC_IN_PLACE` (Yaekai), `VICTORY_THREAD_TICKET` (VIctory-NVIDIA) | | next series | off |

### The scarlett-fox branch (victory-scarlett-3070, cool_fox f4b48ad + Scarlett), merged

Merged on branch `scarlett-merge` (three-way from e12c29f; our translator, cache key, E5 switch
and icon kept; its translator-only features answered "absent" in `shader_compat.rs`). Same
translations (25,782 shaders), same draws a frame (4,350-4,420) and the same draws dropped
for want of a host pipeline (250-300 a frame, as on master). Cell, half scale, warm caches,
frame times from 160 s, three alternating pairs: master 24.1 fps median (23.7 mean), merge
24.7 (24.2): about 2.5 % faster, 400 MB less RAM. Its launcher's nine performance switches
together (TABLE_MEMO, MAP_LOCK_FINE, PARTIAL/BAKE_DELTA, POOL_DISCARD, FILL_TITLE, POOL_CAP,
LIST_ALIAS_EXACT, LIST_HOLD): 21.6 median, no better. Its own large gains were against
cool_fox with `VICTORY_LEVELS_ON_DEMAND=1`, which was already off here. Not adopted as the
base; the branch is kept.

### Where the wall is now (2026-11-26 night)

Cell, half scale, warm: 24-25 fps steady. `VICTORY_THIN_THREADS=4` 24.7 / 24.8 against 6's 24.6
(no change). `VICTORY_RENDER_SCALE=0.4` (FSR) 23.5: no faster than 0.5, the card drops to
82 % busy and CPU preparation rises to 35 ms. Below half scale the submitting thread is the
limit. Its samples (kitty6-prof with the PDB, 40 s, busy 78 %): `GetWriteWatch` 11 %
(`watch::kernel_written` <- `ask_run` <- `ask_ahead`), a wait in a descriptor guard's drop 8 %
(the heap-sync thread mirroring the title's descriptor records, about 15 ms a frame of its
own), `shadow::wide` 20 % inclusive (fanning copies out and waiting for them), a `Vec::retain`
in `card_before` 5 %. These are the next targets for real gains past 25 fps.

### Profile-guided optimization (2026-11-26 night)

`cf-bench\pgo.ps1`: instrumented build (`--features pgo`, profile written every 15 s), trained
on the walking route (330 s, 17 profiles, 20 MB), merged, rebuilt with `-Cprofile-use`. Cell,
half scale, warm caches, two pairs, steady state: plain 24.4 / 24.1 fps median, PGO 23.4 /
24.1. No gain while the card is 84-91 % busy; not adopted (the pipeline is kept).

### Profiles (2026-11-26)

- Card stamps per pass (`VICTORY_CARD_SPANS=1`, now in release builds), cell, full quality: 16
  submissions in 39 `ExecuteCommandLists` calls a frame; the card works about 46 ms: 42 on the
  title's lists, about 4 on our compares (with the tail of the title's work), 3.4 on our
  rehash lists (with uploads); idle between submissions 7-10 ms.
- Frame times (`VICTORY_FRAME_TIMES`), full quality on the 420 s route: median 19.3 fps but 1 %
  lows 4.7 fps, 267 frames over 100 ms; frames spread 40-110 ms in the corridors (mean 65 ms),
  not single spikes.
- CPU samples (Kitty6's `kitty6-prof`, built from source), half scale through FSR: the
  submitting thread runs 80 % of the time, about a quarter of its samples in the Windows heap
  (allocate, free, its lock) and 7 % in `GetWriteWatch`; the list threads run 36 % and wait
  for work the rest.

Where the time goes (default settings, cell): about 40 ms a frame of CPU preparation, of
it about 20 ms copying 40 MB of shader results back to title memory and 8 ms uploading
56 MB; the card is 87-92 % busy. The submitting thread works 41-42 ms a frame.

With all of the above together (the installed release, `START.bat` defaults): 51.3 ms
(19.5 fps) in the cell, picture correct. The card's clock line then says it is about 97 %
busy, and the copy-back thread waits 25.8 ms a frame for it: the card, not the CPU, now
sets the frame time in the cell.

## victory-nvidia-megafork (rorodev6), compared file by file

It is trustno1-coder's VIctory-NVIDIA (6a31e431) plus four things; against cool_fox about
60 files differ. By the areas looked for:

| Area | megafork | cool_fox / here |
|---|---|---|
| Copy-back | compares on the card from 1 MiB (`CARD_BUFFERS_MIN_MB=1`) | from 64 KiB (`CARD_BUFFERS_MIN_KB=64`), landing on 4 threads: already finer |
| CPU submission | per-draw table walk, heaps set per change | precomputed table plans, cached heap binding, per-host sweeps, async submit, thin lists: ahead |
| Texture read-backs | written by a thread of their own (`VICTORY_TEXBACK_ASYNC`) | **new: ported** |
| Zero-copy | none beyond VIctory | `WINDOWS_BAR_MB`, `LAND_BESIDE` |
| Descriptors | dynamic slots up to half a capped (NVIDIA) heap; `FAIR_DUE` re-check order | 2^18 on a capped heap; no table refused in our runs (no "no free dynamic descriptor slots" line), so not taken |
| NVIDIA shaders | halves (fxceless #927), `NAN_SCRUB`, `CONSTANT_MARKS`, `TARGET_VARIANTS` | paired lanes, local waves: correctness items, kept for the rendering pass |
| VRAM | VIctory's pools | cool_fox's `residency.rs`, pool growth back-off: ahead |
| Async GPU | blocking fence waits not implemented | `WaitOnAddress` fence waits: ahead |
| Compiler | release profile only | `fast` profile (fat LTO, native CPU): ahead |
| Other | pipeline panic recovery (fxceless #1353), mouse input, adapter choice, Windows 10 Agility SDK, intro video flicker fix | panic recovery **ported**; the rest not needed here |

Looked at and not taken: Dolberthi's persistent worker pool (cool_fox spawns threads only
for copies of 32 MB and more, where the spawn is about 1 % of the copy); his smart
copy-back (cool_fox's card-side compare already fetches only changed pieces; skipping the
rest risks stale data the title reads).
