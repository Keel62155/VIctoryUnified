use std::collections::BTreeMap;
use std::path::PathBuf;

type Values = BTreeMap<String, String>;

fn pairs(s: &str) -> Values {
    s.lines()
        .filter_map(|line| line.trim().split_once('='))
        .map(|(key, value)| (key.trim(), value.trim()))
        .filter(|(key, value)| {
            key.starts_with("VICTORY_")
                && key.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
                && !value.contains('\0')
        })
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn overlay(mut saved: Values, external: &Values) -> Values {
    for aliases in [
        &["VICTORY_CONSOLE", "VICTORY_MENU_CONSOLE"][..],
        &["VICTORY_DLSS", "VICTORY_DLSS_MODE", "VICTORY_MENU_DLSS"],
        &["VICTORY_FRAMEGEN", "VICTORY_FRAMEGEN_X", "VICTORY_MENU_FRAMEGEN"],
        &["VICTORY_FG_TYPE", "VICTORY_MENU_FG_TYPE"],
        &["VICTORY_FRAME_INTERVAL_MS", "VICTORY_MENU_FPS"],
        &["VICTORY_MENU_RES", "VICTORY_MENU_X_RES", "VICTORY_MENU_S_RES"],
    ] {
        if aliases.iter().any(|name| external.contains_key(*name)) {
            for name in aliases {
                saved.remove(*name);
            }
        }
    }
    saved.extend(external.clone());
    saved
}

pub fn file() -> Option<PathBuf> {
    let var = |name| std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from);
    var("VICTORY_MENU_FILE")
        .or_else(|| var("VICTORY_DATA_DIR").map(|p| p.join("victory.env")))
        .or_else(|| var("LOCALAPPDATA").map(|p| p.join("VIctoryUnified").join("victory.env")))
}

pub fn apply(cli: &[String], explicit_console: Option<&str>) -> anyhow::Result<()> {
    let mut external = Values::new();
    if let Some(console) = explicit_console {
        external.insert("VICTORY_CONSOLE".into(), console.into());
    }
    for assignment in cli {
        let parsed = pairs(assignment);
        anyhow::ensure!(parsed.len() == 1, "invalid --set value {assignment:?}");
        external.extend(parsed);
    }
    external.extend(std::env::vars().filter_map(|(key, value)| {
        let key = key.to_ascii_uppercase();
        key.starts_with("VICTORY_").then_some((key, value))
    }));
    let path = file();
    let saved = match path.as_ref().map(std::fs::read_to_string) {
        Some(Ok(text)) => pairs(text.trim_start_matches('\u{feff}')),
        Some(Err(e)) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
        _ => Values::new(),
    };
    #[cfg(windows)]
    victory_hle::remember_launch_environment();
    let defaults = external.remove("VICTORY_LAUNCH_DEFAULTS")
        .map_or_else(Values::new, |text| pairs(&text));
    let preferred = overlay(saved.clone(), &external);
    let mut effective = overlay(defaults.clone(), &preferred);
    let (console, mut dlss, fg, frame_cap, res, kind) = {
        let get = |key: &str| effective.get(key).map(String::as_str);
        (
            get("VICTORY_CONSOLE").or_else(|| get("VICTORY_MENU_CONSOLE")).unwrap_or("series-x").to_owned(),
            if get("VICTORY_DLSS") == Some("0") { "off" } else {
                get("VICTORY_DLSS_MODE").or_else(|| get("VICTORY_MENU_DLSS"))
                    .unwrap_or(if get("VICTORY_DLSS") == Some("1") { "quality" } else { "off" })
            }.to_owned(),
            get("VICTORY_FRAMEGEN_X").or_else(|| get("VICTORY_MENU_FRAMEGEN"))
                .unwrap_or(if get("VICTORY_FRAMEGEN") == Some("1") { "2" } else { "0" }).to_owned(),
            get("VICTORY_MENU_FPS").map(str::to_owned),
            if get("VICTORY_CONSOLE").or_else(|| get("VICTORY_MENU_CONSOLE")) == Some("series-s") {
                get("VICTORY_MENU_S_RES").or_else(|| get("VICTORY_MENU_RES").filter(|v| matches!(*v, "1080" | "720" | "626" | "540")))
            } else {
                get("VICTORY_MENU_RES").or_else(|| get("VICTORY_MENU_X_RES"))
            }.map(str::to_owned),
            get("VICTORY_FG_TYPE").or_else(|| get("VICTORY_MENU_FG_TYPE")).unwrap_or("auto").to_owned(),
        )
    };
    let mut derived = Values::new();
    anyhow::ensure!(matches!(console.as_str(), "series-s" | "series-x"), "unsupported console {console}");
    if let Some(res) = &res {
        let series_s = console == "series-s";
        let valid = if series_s {
            matches!(res.as_str(), "1080" | "720" | "626" | "540")
        } else {
            matches!(res.as_str(), "1440" | "1080to1440" | "4k")
        };
        anyhow::ensure!(valid, "unsupported {console} resolution {res}");
        if series_s && !["VICTORY_DLSS", "VICTORY_DLSS_MODE", "VICTORY_MENU_DLSS"].iter().any(|name| effective.contains_key(*name)) {
            dlss = match res.as_str() {
                "1080" => "dlaa",
                "626" => "balanced",
                "540" => "performance",
                _ => "off",
            }.to_owned();
        }
        derived.insert(if series_s { "VICTORY_MENU_S_RES" } else { "VICTORY_MENU_X_RES" }.into(), res.clone());
    }
    anyhow::ensure!(matches!(dlss.as_str(), "off" | "ultraperformance" | "ultra-performance" | "performance" | "balanced" | "quality" | "dlaa"), "unsupported DLSS mode {dlss}");
    anyhow::ensure!(matches!(kind.as_str(), "auto" | "dlss" | "fsr"), "unsupported frame-generation type {kind}");
    if !effective.contains_key("VICTORY_MNK_BINDINGS") {
        if let Ok(exe) = std::env::current_exe() {
            let parent = exe.parent().unwrap_or(std::path::Path::new("."));
            for candidate in [parent.join("bindings.ini"), parent.join("../bindings.ini"), parent.join("../../bindings.ini")] {
                if candidate.is_file() {
                    derived.insert("VICTORY_MNK_BINDINGS".into(), candidate.to_string_lossy().into_owned());
                    break;
                }
            }
        }
    }
    derived.insert("VICTORY_CONSOLE".into(), console.clone());
    derived.insert("VICTORY_MENU_CONSOLE".into(), console.clone());
    if dlss != "off" {
        derived.insert("VICTORY_DLSS".into(), "1".into());
        derived.insert("VICTORY_DLSS_MODE".into(), dlss.clone());
    }
    derived.insert("VICTORY_MENU_DLSS".into(), dlss);
    if effective.contains_key("VICTORY_FRAMEGEN_X") || effective.contains_key("VICTORY_MENU_FRAMEGEN") {
        let n = fg.parse::<u32>().ok().filter(|n| *n == 0 || (2..=6).contains(n))
            .ok_or_else(|| anyhow::anyhow!("invalid frame-generation multiplier {fg}; use 0 or 2–6"))?;
        if kind != "fsr" {
            derived.insert("VICTORY_FRAMEGEN".into(), "1".into());
            derived.insert("VICTORY_FRAMEGEN_X".into(), n.to_string());
        }
    }
    if kind == "fsr" {
        eprintln!("VICTORY_MENU_FG_TYPE=fsr: use the OptiScaler launcher option for FSR frame generation in this build");
    }
    derived.insert("VICTORY_MENU_FG_TYPE".into(), kind);
    derived.insert("VICTORY_MENU_FRAMEGEN".into(), fg);
    if let Some(cap) = frame_cap {
        let n = cap.parse::<u32>().ok().filter(|n| (1..=1000).contains(n))
            .ok_or_else(|| anyhow::anyhow!("invalid FPS cap {cap}; use 1–1000"))?;
        derived.insert("VICTORY_FRAME_INTERVAL_MS".into(), (1000.0 / f64::from(n)).to_string());
    }
    if console == "series-x" {
        match res.as_deref() {
            Some("1440") => { derived.insert("VICTORY_NATIVE_1440P".into(), "1".into()); }
            Some("1080to1440") => { derived.insert("VICTORY_RENDER_1080P".into(), "1".into()); }
            _ => {}
        }
    }
    if let Some(path) = path {
        derived.insert("VICTORY_MENU_FILE".into(), path.to_string_lossy().into_owned());
    }
    for (key, value) in derived {
        effective.entry(key).or_insert(value);
    }
    for (key, value) in effective {
        let source = if external.contains_key(&key) {
            "external"
        } else if saved.get(&key) == Some(&value) {
            "user file"
        } else if defaults.get(&key) == Some(&value) {
            "launcher default"
        } else {
            "derived or default"
        };
        println!("config [{source}] {key}={value:?}");
        // SAFETY: runtime settings are applied before starting the title and its worker threads.
        unsafe { std::env::set_var(key, value) };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_aliases_override_saved_members_of_the_same_group() {
        let file = pairs("# keep\nVICTORY_MENU_DLSS=quality\nVICTORY_DLSS=0\nVICTORY_RELAY=2");
        let ext = pairs("VICTORY_DLSS_MODE=performance");
        assert_eq!(overlay(file, &ext), pairs("VICTORY_DLSS_MODE=performance\nVICTORY_RELAY=2"));
    }
}
