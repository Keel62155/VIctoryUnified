//! What the player chose, kept in `launcher-settings.toml` beside the launcher, and the presets.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A starting point a beginner picks; every setting can still be changed after.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    Balanced,
    MaxPerformance,
    Quality,
    Custom,
}

impl Preset {
    pub const PICKABLE: [Preset; 3] = [Preset::MaxPerformance, Preset::Balanced, Preset::Quality];

    pub fn name(self) -> &'static str {
        match self {
            Preset::Balanced => "Balanced",
            Preset::MaxPerformance => "Max performance",
            Preset::Quality => "Quality",
            Preset::Custom => "Custom",
        }
    }

    /// What the card says (measured on an RTX 4060 8 GB + i7-14700F, 2026-11-27).
    pub fn blurb(self) -> &'static str {
        match self {
            Preset::Balanced => "Default settings with FSR upscaling at half render size.",
            Preset::MaxPerformance => {
                "Half render size with the in-place and fine-grained descriptor sync options enabled."
            }
            Preset::Quality => "The game's 2/3 render scale for a sharper image.",
            Preset::Custom => "Your own mix of the settings below.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scale {
    /// 960x540 upscaled to 1080p (default).
    Half,
    /// The game's own 2/3 (VICTORY_QUALITY=1).
    TwoThirds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Upscaler {
    /// AMD FSR through a locally supplied AMD DLL (the default).
    Fsr,
    /// NVIDIA DLSS through OptiScaler (downloaded and set up on first use).
    DlssOptiScaler,
    /// Direct NVIDIA Streamline upscaling.
    DlssNative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameGen {
    Off,
    /// OptiScaler's frame generation (FSR frame generation, 2x); End toggles it in game.
    OptiScaler,
    /// NVIDIA Smooth Motion, the driver's own (needs administrator rights to turn on).
    SmoothMotion,
    /// NVIDIA DLSS Frame Generation through Streamline (RTX 40/50, 2x), on our present; FSR
    /// keeps upscaling. Streamline is downloaded and checked on first use.
    DlssStreamline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Agility {
    /// Windows 10: the Agility SDK; Windows 11: Windows' own Direct3D 12.
    Auto,
    On,
    Off,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub game_dir: String,
    pub preset: Preset,
    pub scale: Scale,
    pub upscaler: Upscaler,
    pub dlss_mode: String,
    pub dlss_model: String,
    pub dlss_fg_multiplier: u32,
    pub dlss_fg_at_start: bool,
    pub framegen: FrameGen,
    /// OptiScaler's frame generation on from the start (otherwise End turns it on).
    pub framegen_at_start: bool,
    /// Menus and videos at 60 fps (the game's own 30 otherwise).
    pub menus_60: bool,
    pub sync_in_place: bool,
    pub sync_fine: bool,
    /// Threads that translate draw calls (the tested best here: 6).
    pub draw_threads: u32,
    /// Landing threads (0: the build's own choice).
    pub landing_threads: u32,
    pub agility: Agility,
    pub raytracing: bool,
    /// More environment variables, one `NAME=value` a line (for testers).
    pub extra_env: String,
}

impl Default for Settings {
    fn default() -> Self {
        let mut s = Settings {
            game_dir: String::new(),
            preset: Preset::Balanced,
            scale: Scale::Half,
    upscaler: Upscaler::Fsr,
            dlss_mode: "quality".into(),
            dlss_model: "default".into(),
            dlss_fg_multiplier: 2,
            dlss_fg_at_start: true,
            framegen: FrameGen::Off,
            framegen_at_start: false,
            menus_60: true,
            sync_in_place: false,
            sync_fine: false,
            draw_threads: 6,
            landing_threads: 0,
            agility: Agility::Auto,
            raytracing: false,
            extra_env: String::new(),
        };
        s.apply(Preset::Balanced);
        s
    }
}

impl Settings {
    /// Sets what `preset` decides; the rest (game folder, upscaler, frame generation) stays.
    pub fn apply(&mut self, preset: Preset) {
        let (scale, in_place, fine) = match preset {
            Preset::Balanced => (Scale::Half, false, false),
            Preset::MaxPerformance => (Scale::Half, true, true),
            Preset::Quality => (Scale::TwoThirds, false, false),
            Preset::Custom => return,
        };
        self.scale = scale;
        self.sync_in_place = in_place;
        self.sync_fine = fine;
        self.draw_threads = 6;
        self.landing_threads = 0;
        self.raytracing = false;
        self.preset = preset;
    }

    /// The preset these settings are, or Custom.
    pub fn matching_preset(&self) -> Preset {
        Preset::PICKABLE
            .into_iter()
            .find(|&p| {
                let mut s = self.clone();
                s.apply(p);
                s.scale == self.scale
                    && s.sync_in_place == self.sync_in_place
                    && s.sync_fine == self.sync_fine
                    && s.draw_threads == self.draw_threads
                    && s.landing_threads == self.landing_threads
                    && s.raytracing == self.raytracing
            })
            .unwrap_or(Preset::Custom)
    }

    pub fn file(root: &Path) -> PathBuf {
        root.join("launcher-settings.toml")
    }

    pub fn load(root: &Path) -> Settings {
        std::fs::read_to_string(Self::file(root))
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, root: &Path) -> std::io::Result<()> {
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(Self::file(root), text)
    }

    /// The `NAME=value` lines of [`Settings::extra_env`] that parse.
    pub fn extra(&self) -> Vec<(String, String)> {
        self.extra_env
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }
                let (k, v) = line.split_once('=')?;
                let k = k.trim();
                (!k.is_empty() && !k.contains(char::is_whitespace))
                    .then(|| (k.to_string(), v.trim().to_string()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_round_trip_and_changes_make_custom() {
        let mut s = Settings::default();
        assert_eq!(s.matching_preset(), Preset::Balanced);
        s.apply(Preset::MaxPerformance);
        assert_eq!(s.matching_preset(), Preset::MaxPerformance);
        s.draw_threads = 8;
        assert_eq!(s.matching_preset(), Preset::Custom);
        let text = toml::to_string_pretty(&s).unwrap();
        assert_eq!(toml::from_str::<Settings>(&text).unwrap(), s);
    }

    #[test]
    fn extra_lines_parse() {
        let s = Settings {
            extra_env: "# note\nVICTORY_LOG=info\n bad line\nA = 1 \n".into(),
            ..Settings::default()
        };
        assert_eq!(
            s.extra(),
            vec![
                ("VICTORY_LOG".into(), "info".into()),
                ("A".into(), "1".into())
            ]
        );
    }
}
