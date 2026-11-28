@echo off
rem VIctoryUnified v2.0 command-line fallback.
rem START.bat [game-folder] plays; START.bat warm prepares the shader cache.
setlocal EnableExtensions DisableDelayedExpansion
set "here=%~dp0"
if /i "%~1"=="--help" (
    echo Usage: START.bat [game-folder] or START.bat warm
    echo Set VICTORY_GAME_DIR to supply the game folder without a prompt.
    exit /b 0
)
set "runtime_home=%LOCALAPPDATA%\VIctoryUnified\v2.0"
set "runner=%here%target\release\victory.exe"
if defined VICTORY_RUNNER set "runner=%VICTORY_RUNNER%"
if not exist "%runner%" (
    echo Run cargo build --workspace --locked --release first. 1>&2
    exit /b 2
)
if not defined VICTORY_GAME_DIR if exist "%~1\game_scarlett_master.exe" set "VICTORY_GAME_DIR=%~1"
if not defined VICTORY_GAME_DIR set /p "VICTORY_GAME_DIR=Path to your game folder: "
set "VICTORY_GAME_DIR=%VICTORY_GAME_DIR:"=%"
if not exist "%VICTORY_GAME_DIR%\game_scarlett_master.exe" (
    echo Set VICTORY_GAME_DIR to the folder containing game_scarlett_master.exe. 1>&2
    exit /b 2
)
if not defined VICTORY_CACHE_DIR set "VICTORY_CACHE_DIR=%runtime_home%\cache"
if not defined VICTORY_DATA_DIR set "VICTORY_DATA_DIR=%runtime_home%\data"
if not exist "%runtime_home%\logs" mkdir "%runtime_home%\logs"

rem (The shader warm-up comes after every setting below: the switches that change a
rem translation are part of the cache's key, so a warm-up without them fills a cache that play
rem never reads.)
rem --- codec_d's pareto-residency set (Play-VIctory.cmd) ---
rem (VICTORY_BARE=1 skips this set: a diagnostic, the runtime's own defaults)
if defined VICTORY_BARE goto pareto_done
if not defined VICTORY_AMBIENT_AIR set "VICTORY_AMBIENT_AIR=1.5"
if not defined VICTORY_TEXTURE_NO_WAIT set "VICTORY_TEXTURE_NO_WAIT=1"
if not defined VICTORY_RELAY set "VICTORY_RELAY=2"
if not defined VICTORY_WIDE_BACK set "VICTORY_WIDE_BACK=1"
if not defined VICTORY_CARD_REHASH_RUNS set "VICTORY_CARD_REHASH_RUNS=1"
if not defined VICTORY_JOIN_COPIES set "VICTORY_JOIN_COPIES=1"
if not defined VICTORY_INDIRECT_WINDOW set "VICTORY_INDIRECT_WINDOW=1"
if not defined VICTORY_RELAY_COPY set "VICTORY_RELAY_COPY=1"
if not defined VICTORY_CARD_STEPS set "VICTORY_CARD_STEPS=2"
if not defined VICTORY_INDIRECT_BATCH set "VICTORY_INDIRECT_BATCH=2"
if not defined VICTORY_MAP_THREADS set "VICTORY_MAP_THREADS=1"
if not defined VICTORY_CARD_AWAKE set "VICTORY_CARD_AWAKE=1"
if not defined VICTORY_SHOW_AFTER set "VICTORY_SHOW_AFTER=1"
if not defined VICTORY_AMBIENT set "VICTORY_AMBIENT=0.8"
if not defined VICTORY_THIN set "VICTORY_THIN=1"
if not defined VICTORY_JOIN_PARTS set "VICTORY_JOIN_PARTS=8"
if not defined VICTORY_UPLOAD_AHEAD set "VICTORY_UPLOAD_AHEAD=1"
rem VICTORY_LEVELS_ON_DEMAND=1 (codec_d's set) is left out: measured here, it made rooms dark,
rem characters black silhouettes and light fall off in rings (prison cell, same spot, same
rem speed without it: 53 ms against 52 ms a frame).
if not defined VICTORY_LEVELS_ON_DEMAND set "VICTORY_LEVELS_ON_DEMAND=0"
if not defined VICTORY_CARD_BUFFERS_MIN_KB set "VICTORY_CARD_BUFFERS_MIN_KB=64"
:pareto_done
rem --- 16-bit float results rounded to nearest as the console rounds them, not toward zero as
rem NVIDIA's conversion does: closer to the console, at no measured cost (50.5 ms). It did not
rem by itself fix the banding (see VICTORY_WAVE64_PAIRED below). Part of the shader cache key.
if not defined VICTORY_F16_NEAREST set "VICTORY_F16_NEAREST=1"
rem --- the colour banding and blotch fix: the title's 64-lane GPU programs translated exactly
rem (two console lanes per NVIDIA thread, ported from kelvaroq) instead of in an approximate
rem form whose per-half results were wrong (banding, blotches) or not at all (black squares on
rem faces, blocky hair). Measured: no banding, no blocks, 46-49 ms a frame in the corridor.
if not defined VICTORY_WAVE64_PAIRED set "VICTORY_WAVE64_PAIRED=1"
rem --- measured on an RTX 4060 8 GB, same scene, fresh checkpoint each run: 1.5 GB of
rem the card's memory for the game's buffers took the prison cell from 107-112 ms a frame to
rem 49-69 ms (the default under 12 GB is 512 MB, at which the card reads them over the bus).
if not defined VICTORY_CARD_BUFFERS_MB set "VICTORY_CARD_BUFFERS_MB=1536"
rem --- Resizable BAR (turn it on in your BIOS; CHECK-REBAR.bat shows whether it is on): 1 GB of the
rem title's own allocations written by the CPU straight into the card's memory. Measured at the
rem same spot: 59.8 -> 48.8 ms a frame. Without Resizable BAR it does nothing.
if not defined VICTORY_WINDOWS_BAR_MB set "VICTORY_WINDOWS_BAR_MB=1024"
rem --- the title's R9G9B9E5 lighting targets as 32-bit R11G11B10_FLOAT (from the "willy" build)
rem instead of 64-bit RGBA16F: half the traffic. Measured in the cell: 60.2 -> 51.2 ms, no banding.
if not defined VICTORY_E5_TARGETS set "VICTORY_E5_TARGETS=r11g11b10"
rem --- 6 threads translating draw calls (cool_fox's default is 8): fewer leave the P-cores to the
rem submitting thread. Measured in the cell: 8 -> 60.2 ms, 6 -> 50.2, 12 -> 63.1, 16 -> 71.5.
if not defined VICTORY_THIN_THREADS set "VICTORY_THIN_THREADS=6"
rem --- texture read-backs written where they land (VICTORY_TEXBACK_ASYNC=1, ported from
rem victory-nvidia-megafork, turned rooms purple and blotchy here).
if not defined VICTORY_TEXBACK_ASYNC set "VICTORY_TEXBACK_ASYNC=0"
rem --- for NVIDIA: the title's Series S upscale runs through AMD's PC FSR DLL (fsr\, AMD-signed,
rem redistributed under AMD's licence in fsr\): the title's own upscaler shader does not translate
rem for NVIDIA cards, and without it the picture stays black. VICTORY_FSR_DLL: another copy.
if not defined VICTORY_FSR_DLL if exist "%runtime_home%\fsr\amd_fidelityfx_upscaler_dx12.dll" set "VICTORY_FSR_DLL=%runtime_home%\fsr\amd_fidelityfx_upscaler_dx12.dll"
if not defined VICTORY_FSR_DLL echo WARNING: fsr\amd_fidelityfx_upscaler_dx12.dll is missing: the picture may stay black. 1>&2
if not defined VICTORY_LOG if not defined RUST_LOG set "VICTORY_LOG=warn"
rem Your own window icon, if you like: put a .ico at icon\game.ico (none is shipped).
rem --- the 3D world drawn at half width and height (960x540) and upscaled to 1080p by FSR, as the
rem default 2/3 is: measured in the cell, steady 22-23 fps against about 19.5 at 2/3, picture right.
rem VICTORY_QUALITY=1 keeps the title's own 2/3 scale.
if not defined VICTORY_QUALITY if not defined VICTORY_RENDER_SCALE set "VICTORY_RENDER_SCALE=0.5"
if /i "%~1"=="rt" (set "VICTORY_BVH_BUILDER=1" & set "VICTORY_BVH_SIZES_WAIT=1")
rem --- 60 frames a second where the game can (menus, videos; from scarlet-fps): our frame clock at
rem 16.667 ms and the game's own 30 fps setting read as 60 (patched in memory, only in the build of
rem the game it was made for). VICTORY_FRAME_INTERVAL_MS=33.333 before starting: 30 as before.
if not defined VICTORY_FRAME_INTERVAL_MS set "VICTORY_FRAME_INTERVAL_MS=16.667"
rem --- Windows 10: Microsoft's DirectX 12 Agility SDK beside the game runtime, downloaded once from
rem nuget.org and checked (tools\setup-agility.ps1): Windows 10's own Direct3D 12 refuses the game's
rem shaders. Windows 11 keeps its own runtime (VICTORY_AGILITY=1: the SDK there too). If it cannot
rem be set up, Windows' own Direct3D 12 is used. VICTORY_AGILITY=0: never.
for %%I in ("%runner%") do set "runnerdir=%%~dpI"
if /i not "%VICTORY_AGILITY%"=="0" if not exist "%runnerdir%D3D12\D3D12Core.dll" powershell -NoProfile -ExecutionPolicy Bypass -File "%here%tools\setup-agility.ps1" -Bin "%runnerdir%."
if /i "%~1"=="warm" goto warm
if exist "%VICTORY_CACHE_DIR%\shaders" goto play
:warm
echo Translating every shader into the cache with 6 workers (a few minutes, once per build)...
"%runner%" shaders --jobs 6 "%VICTORY_GAME_DIR%"
if errorlevel 1 exit /b %errorlevel%
if /i "%~1"=="warm" exit /b 0
:play
rem Only with "autokeys" (testing): holds Enter (= A) for 1 s every 4 s from 15 s to 151 s to get
rem through the start screens (at 60 fps the menus let 0.2 s taps pass unseen). Off by default: the
rem player confirms the start screens and legal notices themselves.
if /i "%~1"=="autokeys" if not defined VICTORY_KEYS call :keys
for /f "tokens=1-3 delims=/:. " %%a in ("%time: =0%") do set "stamp=%%a%%b%%c"
set "log=%runtime_home%\logs\run-%date:~-4%%date:~4,2%%date:~7,2%-%stamp%.log"
echo Starting. Log: %log%
if not defined VICTORY_CONSOLE set "VICTORY_CONSOLE=series-s"
"%runner%" run "%VICTORY_GAME_DIR%\game_scarlett_master.exe" --console %VICTORY_CONSOLE% --execute 2> "%log%"
exit /b %errorlevel%

:keys
set "VICTORY_KEYS="
for /l %%s in (15,4,151) do call :key %%s
exit /b 0

:key
set /a "until=%~1+1"
set "VICTORY_KEYS=%VICTORY_KEYS%%~1-%until%:Enter,"
exit /b 0
