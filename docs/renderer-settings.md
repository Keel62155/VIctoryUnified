# Renderer settings

Use one assignment per line in `victory.env`. Lines beginning with `#` are comments. Set `VICTORY_MENU_FILE` to choose the file. With the graphical launcher the file is under the preview data directory, at `data/victory.env`. External environment variables and external CLI switches outrank saved settings, followed by runtime defaults. An external GUI preset can therefore supersede an F1 value on the next launch. `victory.exe config` prints the resolved configuration without starting a game.

Open F1 with the keyboard or press Menu + View together on the first XInput controller. Use the arrows or D-pad to navigate and adjust. Enter or A selects, and Escape or B backs out. Settings marked live apply immediately. Native FG can switch live in F1 after the Streamline swap chain has been initialized. F3 toggles the stats overlay.

## F1 settings

| Setting | Meaning | Menu values | Runtime default or policy | Applies |
| --- | --- | --- | --- | --- |
| `VICTORY_MENU_DLSS` | Dlss | `quality`, `balanced`, `performance`, `ultraperformance`, `dlaa`, `off` | off | Restart |
| `VICTORY_MENU_FRAMEGEN` | Frame generation | `0`, `2`, `3`, `4`, `5`, `6` | 0 | Live when initialized |
| `VICTORY_MENU_FG_TYPE` | Frame gen type | `auto`, `dlss`, `fsr` | auto | Restart |
| `VICTORY_MENU_FPS` | Frame rate cap | `1000`, `120`, `60`, `30` | engine or launcher preset | Restart |
| `VICTORY_FPS_LINE` | Fps at the top | `1`, `0` | 1 | Live |
| `VICTORY_PANEL` | Stats panel (f3) | `0`, `1` | 0 | Live |
| `VICTORY_VSYNC` | Vsync | `0`, `1` | 0 | Live |
| `VICTORY_FULLSCREEN` | Fullscreen (no border) | empty, 1 | engine or launcher preset | Restart |
| `VICTORY_MOUSE_CAPTURE` | Mouse capture | empty or 0: free cursor, 1: capture | engine or launcher preset | Live (outside F1) |
| `VICTORY_MNK` | Mouse and keyboard | empty, 1 | engine or launcher preset | Restart |
| `VICTORY_IME_ENGLISH` | English keyboard | empty, 1 | engine or launcher preset | Restart |
| `VICTORY_MENU_CONSOLE` | Console | `series-x`, `series-s` | series-x | Restart |
| `VICTORY_MENU_RES` | Series x picture | `1440`, `1080to1440`, `4k` | 4k | Restart |
| `VICTORY_MENU_S_RES` | Series s picture | `1080`, `720`, `626`, `540` | 720 | Restart |
| `VICTORY_DLSS_PRESET` | Dlss model | `empty`, `k`, `j`, `m`, `l` | engine or launcher preset | Restart |
| `VICTORY_AMBIENT` | Light estimate | `0.8`, `1`, `0.5`, `empty` | engine or launcher preset | Restart |
| `VICTORY_AMBIENT_SPECULAR` | Reflection estimate | 1, empty | engine or launcher preset | Restart |
| `VICTORY_AMBIENT_AIR` | Fog cap | `1.5`, `1`, `2`, `empty` | engine or launcher preset | Restart |
| `VICTORY_BVH_BUILDER` | Ray tracing (slow, broken) | empty, 1 | engine or launcher preset | Restart |
| `VICTORY_WATER_LIGHT` | Water light | `empty`, `0` | engine default | Restart |
| `VICTORY_LEAVE_OUT` | Heat haze and decal defrag | `02384a60fda6f2d0,731a39171b568298`, `empty` | engine or launcher preset | Restart |
| `VICTORY_EARLY_DEPTH` | Early depth test | 1, empty | engine or launcher preset | Restart |
| `VICTORY_WC_NO_BACK` | Skip big copy-backs | 1, empty | engine or launcher preset | Restart |
| `VICTORY_SOFT_VIEWS` | Soft views (no cut freezes) | `copies`, `empty` | engine or launcher preset | Restart |
| `VICTORY_TEXTURE_NO_WAIT` | Texture no wait | 1, empty | engine or launcher preset | Restart |
| `VICTORY_RELAY` | Relay (speed, triangles) | 2, 1, 0 | engine or launcher preset | Restart |
| `VICTORY_CARD_STEPS` | Card steps | 2, 1, 0 | engine or launcher preset | Restart |
| `VICTORY_INDIRECT_BATCH` | Indirect batch | 2, 1, 0 | engine or launcher preset | Restart |
| `VICTORY_CARD_BUFFERS_MIN_KB` | Card compares from (kb) | `64`, `1024` | engine or launcher preset | Restart |
| `VICTORY_PORT_CONSTANTS` | Constants heap in place | empty, 1 | engine or launcher preset | Restart |
| `VICTORY_THIN` | Thin lists (heads may be missing) | 1, empty | engine or launcher preset | Restart |
| `VICTORY_JOIN_PARTS` | Join parts | `8`, `empty` | engine or launcher preset | Restart |
| `VICTORY_ASYNC_SUBMIT` | Async submit (frame gen without thin) | empty, 1 | engine or launcher preset | Restart |

## Launcher and native NVIDIA switches

- `VICTORY_DLSS` enables native upscaling with `1`. `VICTORY_DLSS_MODE` also accepts `off`, `dlaa`, `quality`, `balanced`, `performance`, and `ultraperformance`. The default mode is Quality if DLSS is explicitly enabled.
- `VICTORY_DLSS_PRESET` defaults to automatic selection and accepts Default, E, F, J, K, L and M. The installed SDK determines whether a preset is available.
- `VICTORY_FRAMEGEN=1` prepares native DLSS FG. `VICTORY_FRAMEGEN_X` sets 0 for off or 2–6, clamped to the SDK-reported limit. Use 2× on RTX 40-series. `VICTORY_FRAMEGEN_FOCUSED_ONLY=1` generates only while the game has focus.
- `VICTORY_STREAMLINE_DIR` points to a local NVIDIA SDK runtime. The GUI can obtain Streamline from NVIDIA when native DLSS is selected.
- `VICTORY_STREAMLINE_PRESENT=1` requests the Streamline swap-chain proxy for upscaling alone. `VICTORY_REFLEX_SLEEP=1` enables optional Reflex sleep. `VICTORY_FRAMEGEN_HUDLESS=0` disables the separate HUD-less color copy.
- `VICTORY_CONSOLE` aliases `VICTORY_MENU_CONSOLE`. `VICTORY_FRAME_INTERVAL_MS` is the numeric frame interval used by the host and can override the saved F1 cap.
- `VICTORY_DATA_DIR` and `VICTORY_CACHE_DIR` override the profile and cache directories. Use the preview defaults for initial testing. Setting `VICTORY_CACHE_DIR` explicitly can reuse a compatible cache. Shader warming uses six workers.
- `VICTORY_MNK_BINDINGS` points to the bindings file for the optional mouse-to-controller mapping. The bundled profile is seeded once and can be edited in F1.
- `VICTORY_SCRIPTHOOK=1` opts into an installed ScriptHook VI host. See [ScriptHook host setup](scripthook-host.md).

Use the GUI’s OptiScaler path for optional FSR frame generation. The standalone F1 FSR 3 selection reports a warning in this preview. The unofficial NVIDIA multiplier unlocker is excluded.
