//! Starts the locally built runtime directly for gameplay and shader translation.

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

use crate::settings::{Agility, FrameGen, Scale, Settings, Upscaler};

/// A separate console for the native game runtime.
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
/// No window for helper processes whose output the launcher shows itself.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// What the background work tells the window.
pub enum Event {
    Line(String),
    GameEnded(Option<i32>),
    ToolDone,
}

/// Whether `dir` is a game folder (holds `game_scarlett_master.exe`).
pub fn is_game_dir(dir: &str) -> bool {
    !dir.trim().is_empty()
        && Path::new(dir.trim())
            .join("game_scarlett_master.exe")
            .is_file()
}

/// The environment the game is started with, for these settings (OptiScaler's own variables come
/// from [`optiscaler`]).
pub fn environment(s: &Settings) -> Vec<(String, String)> {
    let mut env: Vec<(String, String)> = Vec::new();
    let mut set = |k: &str, v: &str| env.push((k.to_string(), v.to_string()));
    set("VICTORY_GAME_DIR", s.game_dir.trim());
    if s.scale == Scale::TwoThirds {
        set("VICTORY_QUALITY", "1");
    }
    if !s.menus_60 {
        set("VICTORY_FRAME_INTERVAL_MS", "33.333");
    }
    if s.sync_in_place {
        set("VICTORY_SYNC_IN_PLACE", "1");
    }
    if s.sync_fine {
        set("VICTORY_SYNC_FINE", "1");
    }
    set(
        "VICTORY_THIN_THREADS",
        &s.draw_threads.clamp(1, 32).to_string(),
    );
    if s.landing_threads > 0 {
        set(
            "VICTORY_LAND_THREADS",
            &s.landing_threads.clamp(1, 16).to_string(),
        );
    }
    match s.agility {
        Agility::Auto => {}
        Agility::On => set("VICTORY_AGILITY", "1"),
        Agility::Off => set("VICTORY_AGILITY", "0"),
    }
    if s.raytracing {
        // As START-RAYTRACING.bat: the ray-traced shadows need the title's own render scale.
        set("VICTORY_BVH_BUILDER", "1");
        set("VICTORY_BVH_SIZES_WAIT", "1");
        set("VICTORY_NO_WATCHDOG", "1");
        set("VICTORY_QUALITY", "1");
    }
    for (k, v) in s.extra() {
        env.push((k, v));
    }
    env
}

/// Whether OptiScaler is needed (DLSS through it, or its frame generation). Not with DLSS
/// Frame Generation through Streamline: OptiScaler stands in for the system's DXGI, which
/// Streamline's proxies must not meet (FSR upscales then).
pub fn needs_optiscaler(s: &Settings) -> bool {
    s.framegen != FrameGen::DlssStreamline
        && (s.upscaler == Upscaler::DlssOptiScaler || s.framegen == FrameGen::OptiScaler)
}

/// Sets Streamline up (`tools\setup-streamline.ps1`: downloaded and checked on first use);
/// whether DLSS Frame Generation can be asked for.
fn streamline(root: &Path, out: &Sender<Event>) -> bool {
    let script = root.join("tools").join("setup-streamline.ps1");
    if !script.is_file() {
        let _ = out.send(Event::Line(
            "tools\\setup-streamline.ps1 is missing: starting without DLSS Frame Generation."
                .into(),
        ));
        return false;
    }
    let _ = out.send(Event::Line(
        "Checking NVIDIA Streamline (the first time it is downloaded, 276 MB)...".into(),
    ));
    let result = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(&script)
        .current_dir(root)
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    match result {
        Ok(o) => {
            for line in String::from_utf8_lossy(&o.stdout)
                .lines()
                .chain(String::from_utf8_lossy(&o.stderr).lines())
            {
                if !line.trim().is_empty() {
                    let _ = out.send(Event::Line(line.to_string()));
                }
            }
            o.status.success()
        }
        Err(e) => {
            let _ = out.send(Event::Line(format!(
                "Streamline setup could not run ({e}): starting without DLSS Frame Generation."
            )));
            false
        }
    }
}

/// Sets OptiScaler up (`tools\setup-dlss.ps1`: downloaded and checked on first use) and gives the
/// variables that start the game through it; `None` (and why, to `out`) if it cannot be: the game
/// then starts with FSR as before.
fn optiscaler(root: &Path, s: &Settings, out: &Sender<Event>) -> Option<Vec<(String, String)>> {
    let script = root.join("tools").join("setup-dlss.ps1");
    if !script.is_file() {
        let _ = out.send(Event::Line(
            "tools\\setup-dlss.ps1 is missing: starting with FSR.".into(),
        ));
        return None;
    }
    let upscaler = if s.upscaler == Upscaler::DlssOptiScaler {
        "dlss"
    } else {
        "fsr31"
    };
    let mut args = [
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
    ]
    .map(str::to_owned)
    .to_vec();
    args.push(script.display().to_string());
    args.extend(["-Upscaler".into(), upscaler.into()]);
    if s.framegen == FrameGen::OptiScaler && s.framegen_at_start {
        args.push("-FrameGen".into());
    }
    let _ = out.send(Event::Line(format!(
        "Setting up OptiScaler ({upscaler})..."
    )));
    let result = Command::new("powershell")
        .args(&args)
        .current_dir(root)
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    let output = match result {
        Ok(o) => o,
        Err(e) => {
            let _ = out.send(Event::Line(format!(
                "OptiScaler setup could not run ({e}): starting with FSR."
            )));
            return None;
        }
    };
    for line in String::from_utf8_lossy(&output.stdout)
        .lines()
        .chain(String::from_utf8_lossy(&output.stderr).lines())
    {
        if !line.trim().is_empty() {
            let _ = out.send(Event::Line(line.to_string()));
        }
    }
    match output.status.code() {
        Some(0) => {}
        Some(3) => {
            let _ = out.send(Event::Line(
                "No DLSS library found on this PC: starting with FSR.".into(),
            ));
            return None;
        }
        code => {
            let _ = out.send(Event::Line(format!(
                "OptiScaler setup failed (code {code:?}): starting with FSR."
            )));
            return None;
        }
    }
    let dlss = root.join("dlss");
    Some(vec![
        (
            "VICTORY_RUNNER".into(),
            dlss.join("victory.exe").display().to_string(),
        ),
        (
            "VICTORY_FSR_DLL".into(),
            dlss.join("amd_fidelityfx_dx12.dll").display().to_string(),
        ),
        ("VICTORY_FG_CAPTION".into(), "1".into()),
    ])
}

/// The game's executable the launcher runs its own commands with.
fn runner(root: &Path) -> PathBuf {
    root.join("bin").join("victory.exe")
}

pub fn verify(root: &Path) -> io::Result<()> {
    for args in [["run", "--help"], ["shaders", "--help"]] {
        let result = Command::new(runner(root))
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;
        if !result.status.success() {
            return Err(io::Error::other(
                String::from_utf8_lossy(&result.stderr).into_owned(),
            ));
        }
        let stdout = String::from_utf8_lossy(&result.stdout);
        let option = if args[0] == "run" {
            "--execute"
        } else {
            "--jobs"
        };
        if !stdout.contains(option) {
            return Err(io::Error::other(format!(
                "Runtime is missing {} {option}",
                args[0]
            )));
        }
    }
    println!("Runtime verified in {}", root.display());
    Ok(())
}
/// Whether this install's victory.exe knows `smooth-motion` (builds from 2026-11-27 on).
pub fn smooth_motion_available(root: &Path) -> bool {
    Command::new(runner(root))
        .args(["smooth-motion", "status"])
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Smooth Motion on (as administrator: the driver takes its API setting only from one) or off.
fn smooth_motion(root: &Path, on: bool, out: &Sender<Event>) -> bool {
    let exe = runner(root);
    let status = if on {
        let _ = out.send(Event::Line(
            "Turning NVIDIA Smooth Motion on (Windows asks for administrator rights)...".into(),
        ));
        let command = format!(
            "try {{ $p = Start-Process -FilePath '{}' -ArgumentList 'smooth-motion','on' -Verb RunAs -Wait -PassThru -WindowStyle Hidden; exit $p.ExitCode }} catch {{ exit 1 }}",
            exe.display().to_string().replace('\'', "''")
        );
        Command::new("powershell")
            .args(["-NoProfile", "-Command", &command])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
    } else {
        Command::new(&exe)
            .args(["smooth-motion", "off"])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
    };
    let ok = status.is_ok_and(|s| s.success());
    let _ = out.send(Event::Line(match (on, ok) {
        (true, true) => "Smooth Motion is on for this session.".into(),
        (true, false) => {
            "Smooth Motion could not be turned on (the prompt was declined?): starting without it."
                .into()
        }
        (false, true) => "Smooth Motion is off again.".into(),
        (false, false) => {
            "Smooth Motion could not be turned off: run `bin\\victory.exe smooth-motion off`."
                .into()
        }
    }));
    ok
}

const DEFAULTS: &[(&str, &str)] = &[
    ("VICTORY_AMBIENT_AIR", "1.5"),
    ("VICTORY_TEXTURE_NO_WAIT", "1"),
    ("VICTORY_RELAY", "2"),
    ("VICTORY_WIDE_BACK", "1"),
    ("VICTORY_CARD_REHASH_RUNS", "1"),
    ("VICTORY_JOIN_COPIES", "1"),
    ("VICTORY_INDIRECT_WINDOW", "1"),
    ("VICTORY_RELAY_COPY", "1"),
    ("VICTORY_CARD_STEPS", "2"),
    ("VICTORY_INDIRECT_BATCH", "2"),
    ("VICTORY_MAP_THREADS", "1"),
    ("VICTORY_CARD_AWAKE", "1"),
    ("VICTORY_SHOW_AFTER", "1"),
    ("VICTORY_AMBIENT", "0.8"),
    ("VICTORY_THIN", "1"),
    ("VICTORY_JOIN_PARTS", "8"),
    ("VICTORY_UPLOAD_AHEAD", "1"),
    ("VICTORY_LEVELS_ON_DEMAND", "0"),
    ("VICTORY_CARD_BUFFERS_MIN_KB", "64"),
];

const FIXES: &[(&str, &str)] = &[
    ("VICTORY_F16_NEAREST", "1"),
    ("VICTORY_WAVE64_PAIRED", "1"),
    ("VICTORY_CARD_BUFFERS_MB", "1536"),
    ("VICTORY_WINDOWS_BAR_MB", "1024"),
    ("VICTORY_E5_TARGETS", "r11g11b10"),
    ("VICTORY_TEXBACK_ASYNC", "0"),
    ("VICTORY_LOG", "warn"),
];

fn runtime(
    root: &Path,
    settings: &Settings,
    runtime: &Path,
    overrides: &[(String, String)],
) -> Command {
    let mut cmd = Command::new(runtime);
    for (name, _) in std::env::vars_os() {
        if name
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("VICTORY_")
        {
            cmd.env_remove(name);
        }
    }
    let extra = settings.extra();
    if !extra
        .iter()
        .any(|(key, value)| key.eq_ignore_ascii_case("VICTORY_BARE") && !value.is_empty())
    {
        cmd.envs(DEFAULTS.iter().copied());
    }
    cmd.current_dir(root)
        .envs(FIXES.iter().copied())
        .env("VICTORY_FRAME_INTERVAL_MS", "16.667")
        .env("VICTORY_RENDER_SCALE", "0.5")
        .env(
            "VICTORY_FSR_DLL",
            std::env::var_os("VICTORY_FSR_DLL")
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join("fsr/amd_fidelityfx_upscaler_dx12.dll")),
        )
        .envs(environment(settings))
        .envs(overrides.iter().cloned())
        .env("VICTORY_GAME_DIR", settings.game_dir.trim())
        .env("VICTORY_CACHE_DIR", root.join("cache"))
        .env("VICTORY_DATA_DIR", root.join("data"))
        .env_remove("VICTORY_CONFIG")
        .env_remove("VICTORY_KEYS")
        .env_remove("VICTORY_AUTO_A")
        .env_remove("VICTORY_AUTO_INPUT");
    if !extra
        .iter()
        .any(|(key, _)| key.eq_ignore_ascii_case("VICTORY_RENDER_SCALE"))
        && (settings.scale == Scale::TwoThirds || settings.raytracing)
    {
        cmd.env_remove("VICTORY_RENDER_SCALE");
    }
    cmd
}

fn log_path(root: &Path, kind: &str) -> io::Result<PathBuf> {
    let log = root.join("logs");
    fs::create_dir_all(&log)?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_millis();
    Ok(log.join(format!("{kind}-{stamp}-{}.log", std::process::id())))
}

fn shaders(root: &Path, settings: &Settings, force: bool, out: &Sender<Event>) -> io::Result<()> {
    let pack = Path::new(settings.game_dir.trim()).join("shaders.rpf");
    let meta = fs::metadata(&pack)
        .map_err(|error| io::Error::new(error.kind(), format!("{}: {error}", pack.display())))?;
    if !meta.is_file() || meta.len() == 0 {
        return Err(io::Error::other(format!(
            "No shader archive at {}",
            pack.display()
        )));
    }
    let exe = runner(root);
    let mut cmd = runtime(root, settings, &exe, &[]);
    let mut stamp = format!(
        "{:?}|{}|{:?}|{:?}\n",
        pack,
        meta.len(),
        meta.modified()?,
        fs::metadata(&exe)?.modified()?
    );
    for (key, value) in cmd.get_envs().filter(|(_, value)| value.is_some()) {
        use std::fmt::Write as _;
        let _ = writeln!(stamp, "{key:?}={value:?}");
    }
    let marker = root.join("cache/shaders-ahead.txt");
    if !force
        && root.join("cache/shaders").is_dir()
        && fs::read_to_string(&marker).is_ok_and(|v| v == stamp)
    {
        return Ok(());
    }
    let log = log_path(root, "warm")?;
    let _ = out.send(Event::Line(format!(
        "Translating {} with 6 workers. Diagnostics: {}",
        pack.file_name().unwrap_or_default().to_string_lossy(),
        log.display()
    )));
    cmd.args(["shaders", "--jobs", "6"])
        .arg(&pack)
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::piped())
        .stderr(File::create(&log)?);
    let mut child = cmd.spawn()?;
    let stream = child.stdout.take().expect("shader output is piped");
    let mut cached_shader_count = None;
    for line in BufReader::new(stream).lines() {
        let line = line?;
        if line.contains("shaders in the cache") {
            cached_shader_count = line
                .split_whitespace()
                .next()
                .and_then(|n| n.parse::<usize>().ok());
        }
        let _ = out.send(Event::Line(line));
    }
    let status = child.wait()?;
    if !status.success() || cached_shader_count == Some(0) {
        return Err(io::Error::other(format!(
            "Shader warm-up ended with {status}, cached shader count {cached_shader_count:?}. See {}",
            log.display()
        )));
    }
    fs::create_dir_all(root.join("cache"))?;
    fs::write(marker, stamp)
}

fn windows_build() -> Option<u32> {
    let reg = PathBuf::from(std::env::var_os("SystemRoot")?).join("System32/reg.exe");
    let value = Command::new(reg)
        .args([
            "query",
            r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "/v",
            "CurrentBuildNumber",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !value.status.success() {
        return None;
    }
    String::from_utf8_lossy(&value.stdout)
        .split_whitespace()
        .last()?
        .parse()
        .ok()
}

fn prepare_agility(root: &Path, settings: &Settings, exe: &Path, out: &Sender<Event>) {
    if settings.agility == Agility::Off
        || (settings.agility == Agility::Auto
            && windows_build().is_some_and(|build| build >= 22000))
    {
        return;
    }
    let bin = exe.parent().unwrap_or(root);
    if bin.join("D3D12/D3D12Core.dll").is_file() {
        return;
    }
    let mut cmd = Command::new("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
    ])
    .arg(root.join("tools/setup-agility.ps1"))
    .arg("-Bin")
    .arg(bin)
    .creation_flags(CREATE_NO_WINDOW);
    if settings.agility == Agility::On {
        cmd.env("VICTORY_AGILITY", "1");
    } else {
        cmd.env_remove("VICTORY_AGILITY");
        // The embedded helper checks the Windows version before downloading.
        cmd.arg("-Quiet");
    }
    if let Ok(output) = cmd.output() {
        for line in String::from_utf8_lossy(&output.stdout)
            .lines()
            .chain(String::from_utf8_lossy(&output.stderr).lines())
        {
            if !line.trim().is_empty() {
                let _ = out.send(Event::Line(line.to_owned()));
            }
        }
    }
}

pub fn play(root: PathBuf, settings: Settings, out: Sender<Event>) {
    std::thread::spawn(move || {
        let result = play_inner(&root, &settings, &out);
        match result {
            Ok(code) => {
                let _ = out.send(Event::GameEnded(code));
            }
            Err(error) => {
                let _ = out.send(Event::Line(error.to_string()));
                let _ = out.send(Event::GameEnded(Some(1)));
            }
        }
    });
}

fn play_inner(root: &Path, s: &Settings, out: &Sender<Event>) -> io::Result<Option<i32>> {
    let _lock = lock_runtime(root)?;
    crate::payload::install(root)?;
    let game = Path::new(s.game_dir.trim()).join("game_scarlett_master.exe");
    if !game.is_file() {
        return Err(io::Error::other(format!(
            "Game executable missing: {}",
            game.display()
        )));
    }
    let mut exe = runner(root);
    prepare_agility(root, s, &exe, out);
    if let Err(error) = shaders(root, s, false, out) {
        let _ = out.send(Event::Line(format!(
            "{error} Starting with on-demand shader translation."
        )));
    }

    let mut overrides = Vec::new();
    if needs_optiscaler(s)
        && let Some(options) = optiscaler(root, s, out)
    {
        exe = root.join("dlss/victory.exe");
        overrides.extend(
            options
                .into_iter()
                .filter(|(name, _)| name != "VICTORY_RUNNER"),
        );
        prepare_agility(root, s, &exe, out);
    }
    if s.framegen == FrameGen::DlssStreamline && streamline(root, out) {
        overrides.push(("VICTORY_FRAMEGEN".into(), "1".into()));
        overrides.push(("VICTORY_THIN".into(), "1".into()));
        let _ = out.send(Event::Line(
            "Requested NVIDIA DLSS Frame Generation (2x). See the run log for driver support."
                .into(),
        ));
    }

    let log = log_path(root, "run")?;
    let file = File::create(&log)?;
    let _ = out.send(Event::Line(format!(
        "Starting the game. Log: {}",
        log.display()
    )));
    let mut cmd = runtime(root, s, &exe, &overrides);
    let console = cmd
        .get_envs()
        .find_map(|(name, value)| {
            name.eq_ignore_ascii_case("VICTORY_CONSOLE")
                .then_some(value)
                .flatten()
        })
        .and_then(|value| value.to_str())
        .unwrap_or("series-s")
        .to_owned();
    cmd.arg("run")
        .arg(&game)
        .args(["--console", &console, "--execute"])
        .creation_flags(CREATE_NEW_CONSOLE)
        .stderr(file.try_clone()?)
        .stdout(file);
    let smooth = s.framegen == FrameGen::SmoothMotion && smooth_motion(root, true, out);
    let result = cmd.status();
    if smooth {
        smooth_motion(root, false, out);
    }
    let status = result?;
    if !status.success() {
        let _ = out.send(Event::Line(format!(
            "Runtime exited with {status}. See {}",
            log.display()
        )));
    }
    Ok(status.code())
}

pub fn warm(root: PathBuf, s: Settings, out: Sender<Event>) {
    std::thread::spawn(move || {
        let result = (|| {
            let _lock = lock_runtime(&root)?;
            crate::payload::install(&root)?;
            prepare_agility(&root, &s, &runner(&root), &out);
            shaders(&root, &s, true, &out)
        })();
        let _ = out.send(Event::Line(match result {
            Ok(()) => "Shader cache ready.".into(),
            Err(error) => error.to_string(),
        }));
        let _ = out.send(Event::ToolDone);
    });
}

fn lock_runtime(root: &Path) -> io::Result<File> {
    fs::create_dir_all(root)?;
    let file = File::options()
        .read(true)
        .append(true)
        .create(true)
        .open(root.join("runtime.lock"))?;
    file.try_lock().map_err(|error| match error {
        std::fs::TryLockError::WouldBlock => io::Error::other(
            "Another standalone launcher task is active. Wait for it or close the game.",
        ),
        std::fs::TryLockError::Error(error) => error,
    })?;
    Ok(file)
}

// Also detect a game left open by an older launcher that does not hold the runtime lock.
fn require_game_closed(root: &Path) -> io::Result<()> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use windows_sys::Win32::Foundation::{ERROR_NO_MORE_FILES, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
    };

    let root = root.canonicalize()?;
    // SAFETY: Check handles before taking ownership and use initialized Win32 output buffers.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let snapshot = OwnedHandle::from_raw_handle(snapshot);
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut more = Process32FirstW(snapshot.as_raw_handle(), &mut entry);
        let mut image = vec![0; 32_768];
        while more != 0 {
            let name_len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..name_len])
                .eq_ignore_ascii_case("victory.exe")
            {
                let process =
                    OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, entry.th32ProcessID);
                if !process.is_null() {
                    let process = OwnedHandle::from_raw_handle(process);
                    let mut len = image.len() as u32;
                    if QueryFullProcessImageNameW(
                        process.as_raw_handle(),
                        0,
                        image.as_mut_ptr(),
                        &mut len,
                    ) != 0
                    {
                        let path = PathBuf::from(OsString::from_wide(&image[..len as usize]));
                        if path
                            .canonicalize()
                            .is_ok_and(|path| path.starts_with(&root))
                        {
                            return Err(io::Error::other(
                                "Close the standalone game before importing a save.",
                            ));
                        }
                    }
                }
            }
            more = Process32NextW(snapshot.as_raw_handle(), &mut entry);
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(ERROR_NO_MORE_FILES as i32) {
            Ok(())
        } else {
            Err(error)
        }
    }
}

/// Import Story Mode slots from a folder or ZIP without running a script.
pub fn import_save(root: PathBuf, source: PathBuf, out: Sender<Event>) {
    std::thread::spawn(move || {
        let result = (|| {
            let _lock = lock_runtime(&root)?;
            require_game_closed(&root)?;
            crate::saves::import(&root, &source)
        })();
        match result {
            Ok(imported) => {
                if imported.player_added {
                    let _ = out.send(Event::Line(
                        "Installed the supplied Player container for this new profile.".into(),
                    ));
                }
                if let Some(backup) = imported.backup {
                    let _ = out.send(Event::Line(format!(
                        "Backed up existing saves to {}",
                        backup.display()
                    )));
                }
                let _ = out.send(Event::Line(format!(
                    "Imported Story Mode slots: {}",
                    imported.slots.join(", ")
                )));
            }
            Err(error) => {
                let _ = out.send(Event::Line(format!("Save import failed: {error}")));
            }
        }
        let _ = out.send(Event::ToolDone);
    });
}

/// Whether Resizable BAR is on: nvidia-smi's BAR1 total (256 MiB: off).
pub fn check_rebar(out: Sender<Event>) {
    std::thread::spawn(move || {
        let smi = std::env::var("SystemRoot")
            .map(|r| PathBuf::from(r).join("System32").join("nvidia-smi.exe"))
            .unwrap_or_default();
        let text = Command::new(smi)
            .args(["-q", "-d", "MEMORY"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default();
        let mut in_bar = false;
        let mut total = None;
        for line in text.lines() {
            if line.contains("BAR1") {
                in_bar = true;
            } else if in_bar && line.trim_start().starts_with("Total") {
                total = line.split(':').nth(1).map(|v| v.trim().to_string());
                break;
            }
        }
        let _ = out.send(Event::Line(match total {
            Some(t) if t.starts_with("256 ") => format!(
                "Resizable BAR looks OFF (BAR1 {t}). Turn it on in your BIOS for a big speed-up."
            ),
            Some(t) => format!("Resizable BAR is ON (BAR1 {t})."),
            None => {
                "Could not read the BAR size (nvidia-smi not found: not an NVIDIA card?).".into()
            }
        }));
        let _ = out.send(Event::ToolDone);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Preset;

    #[test]
    fn standalone_import_reports_completion_and_uses_the_native_save_path() {
        let root = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        let slot = source.path().join("SARC0002~abc");
        fs::create_dir(&slot).unwrap();
        fs::write(slot.join("slot.bin"), b"test save").unwrap();

        let (send, receive) = std::sync::mpsc::channel();
        import_save(root.path().to_path_buf(), source.path().to_path_buf(), send);
        let timeout = std::time::Duration::from_secs(10);
        assert!(matches!(
            receive.recv_timeout(timeout).unwrap(),
            Event::Line(message) if message == "Imported Story Mode slots: SARC0002~abc"
        ));
        assert!(matches!(
            receive.recv_timeout(timeout).unwrap(),
            Event::ToolDone
        ));
    }

    #[test]
    fn runtime_lock_excludes_other_launcher_tasks() {
        let root = tempfile::tempdir().unwrap();
        let lock = lock_runtime(root.path()).unwrap();
        assert!(lock_runtime(root.path()).is_err());
        drop(lock);
        assert!(lock_runtime(root.path()).is_ok());
    }

    #[test]
    fn native_command_uses_portable_state_and_keeps_startup_manual() {
        let s = Settings {
            game_dir: "D:\\game".into(),
            menus_60: false,
            scale: Scale::TwoThirds,
            extra_env: "VICTORY_AUTO_INPUT=Enter\nVICTORY_KEYS=0:Enter\nVICTORY_CACHE_DIR=D:\\game"
                .into(),
            ..Settings::default()
        };
        let root = Path::new("C:/isolated");
        let exe = runner(root);
        let cmd = runtime(root, &s, &exe, &[]);
        let vars = cmd.get_envs().collect::<std::collections::HashMap<_, _>>();
        let get = |name: &str| vars.get(std::ffi::OsStr::new(name)).copied().flatten();
        assert_eq!(cmd.get_program(), exe.as_os_str());
        assert_eq!(
            get("VICTORY_CACHE_DIR"),
            Some(root.join("cache").as_os_str())
        );
        assert_eq!(
            get("VICTORY_FRAME_INTERVAL_MS"),
            Some(std::ffi::OsStr::new("33.333"))
        );
        assert_eq!(get("VICTORY_RENDER_SCALE"), None);
        assert_eq!(get("VICTORY_AUTO_INPUT"), None);
        assert_eq!(get("VICTORY_KEYS"), None);
    }

    #[test]
    fn max_performance_sets_both_sync_switches() {
        let mut s = Settings {
            game_dir: "D:\\game".into(),
            ..Settings::default()
        };
        s.apply(Preset::MaxPerformance);
        let env = environment(&s);
        assert!(env.contains(&("VICTORY_SYNC_IN_PLACE".into(), "1".into())));
        assert!(env.contains(&("VICTORY_SYNC_FINE".into(), "1".into())));
        assert!(env.contains(&("VICTORY_GAME_DIR".into(), "D:\\game".into())));
        assert!(!env.iter().any(|(k, _)| k == "VICTORY_QUALITY"));
    }

    #[test]
    fn menus_at_30_and_quality() {
        let mut s = Settings {
            menus_60: false,
            ..Settings::default()
        };
        s.apply(Preset::Quality);
        let env = environment(&s);
        assert!(env.contains(&("VICTORY_FRAME_INTERVAL_MS".into(), "33.333".into())));
        assert!(env.contains(&("VICTORY_QUALITY".into(), "1".into())));
    }
}
