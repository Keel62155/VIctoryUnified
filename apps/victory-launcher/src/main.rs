//! VIctoryUnified GUI for game settings and tools.

#![windows_subsystem = "windows"]

mod launch;
mod payload;
mod saves;
mod settings;

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui::{self, Color32, RichText};

use launch::Event;
use settings::{Agility, FrameGen, Preset, Scale, Settings, Upscaler};

const ACCENT: Color32 = Color32::from_rgb(0x3d, 0xd6, 0x8c);
const WARN: Color32 = Color32::from_rgb(0xf0, 0xb4, 0x29);
const DIM: Color32 = Color32::from_gray(150);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Play,
    Graphics,
    Performance,
    Tools,
}

struct App {
    root: PathBuf,
    settings: Settings,
    saved: Settings,
    page: Page,
    log: Vec<String>,
    events: Receiver<Event>,
    send: Sender<Event>,
    game_running: bool,
    tool_running: bool,
    smooth_motion_available: bool,
}

impl App {
    fn new(root: PathBuf) -> App {
        let settings = initial_settings(&root);
        let (send, events) = channel();
        let mut log = Vec::new();
        let smooth_motion_available = match payload::install(&root) {
            Ok(()) => launch::smooth_motion_available(&root),
            Err(error) => {
                log.push(format!("The runtime could not be installed: {error}"));
                false
            }
        };
        // `--page play|graphics|performance|tools`: the page it opens on.
        let mut args = std::env::args().skip(1);
        let mut page = Page::Play;
        let mut play_now = false;
        while let Some(arg) = args.next() {
            if arg == "--play" {
                play_now = true;
            } else if arg == "--page" {
                page = match args.next().as_deref() {
                    Some("graphics") => Page::Graphics,
                    Some("performance") => Page::Performance,
                    Some("tools") => Page::Tools,
                    _ => Page::Play,
                };
            }
        }
        let mut app = App {
            root,
            saved: settings.clone(),
            settings,
            page,
            log,
            events,
            send,
            game_running: false,
            tool_running: false,
            smooth_motion_available,
        };
        // `--play`: start with the saved settings at once (a desktop shortcut).
        if play_now {
            if launch::is_game_dir(&app.settings.game_dir) {
                app.start_game();
            } else {
                app.log.push("--play: pick the game folder first.".into());
            }
        }
        app
    }

    fn start_game(&mut self) {
        self.log.push(format!(
            "Play: preset {}, {}",
            self.settings.preset.name(),
            summary(&self.settings)
        ));
        launch::play(self.root.clone(), self.settings.clone(), self.send.clone());
        self.game_running = true;
    }

    fn poll(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::Line(line) => self.log.push(line),
                Event::GameEnded(code) => {
                    self.game_running = false;
                    self.log.push(match code {
                        Some(0) => "The game has ended.".into(),
                        None => "The game closed without an exit code.".into(),
                        Some(c) => format!("The game ended with code {c}."),
                    });
                }
                Event::ToolDone => self.tool_running = false,
            }
        }
        if self.log.len() > 400 {
            self.log.drain(..self.log.len() - 400);
        }
        self.settings.preset = self.settings.matching_preset();
        if self.settings != self.saved {
            if let Err(e) = self.settings.save(&self.root) {
                self.log.push(format!("Settings could not be saved: {e}"));
            }
            self.saved = self.settings.clone();
        }
    }

    fn busy(&self) -> bool {
        self.game_running || self.tool_running
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.label(
            RichText::new("VIctoryUnified")
                .size(22.0)
                .strong()
                .color(ACCENT),
        );
        ui.label(RichText::new("launcher").color(DIM));
        ui.add_space(18.0);
        for (page, name) in [
            (Page::Play, "Play"),
            (Page::Graphics, "Upscaling & frame gen"),
            (Page::Performance, "Performance"),
            (Page::Tools, "Tools"),
        ] {
            let text = RichText::new(name).size(16.0);
            if ui
                .add_sized(
                    [170.0, 34.0],
                    egui::Button::selectable(self.page == page, text),
                )
                .clicked()
            {
                self.page = page;
            }
            ui.add_space(4.0);
        }
        ui.add_space(18.0);
        ui.separator();
        ui.label(RichText::new(format!("Preset: {}", self.settings.preset.name())).color(DIM));
        ui.label(RichText::new(summary(&self.settings)).color(DIM).small());
    }

    fn play_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Play");
        ui.add_space(8.0);
        ui.label(
            RichText::new("Game folder (your own dump, the folder with game_scarlett_master.exe)")
                .strong(),
        );
        ui.horizontal(|ui| {
            ui.add_sized(
                [ui.available_width() - 110.0, 28.0],
                egui::TextEdit::singleline(&mut self.settings.game_dir),
            );
            if ui
                .add_sized([100.0, 28.0], egui::Button::new("Browse..."))
                .clicked()
                && let Some(dir) = rfd::FileDialog::new()
                    .set_title("The folder with game_scarlett_master.exe")
                    .pick_folder()
            {
                self.settings.game_dir = dir.display().to_string();
            }
        });
        let ok = launch::is_game_dir(&self.settings.game_dir);
        if self.settings.game_dir.trim().is_empty() {
            ui.label(
                RichText::new(
                    "Pick your game folder first. Your game files are only read, never changed.",
                )
                .color(WARN),
            );
        } else if !ok {
            ui.label(RichText::new("No game_scarlett_master.exe in that folder.").color(WARN));
        } else {
            ui.label(RichText::new("Game found.").color(ACCENT));
        }
        ui.add_space(14.0);
        ui.label(RichText::new("Preset").strong());
        ui.horizontal_wrapped(|ui| {
            for preset in Preset::PICKABLE {
                let selected = self.settings.preset == preset;
                let frame = egui::Frame::group(ui.style())
                    .fill(if selected {
                        Color32::from_rgb(0x1f, 0x3a, 0x2c)
                    } else {
                        ui.visuals().faint_bg_color
                    })
                    .stroke(egui::Stroke::new(
                        if selected { 2.0_f32 } else { 1.0 },
                        if selected {
                            ACCENT
                        } else {
                            Color32::from_gray(70)
                        },
                    ))
                    .inner_margin(12.0);
                let response = frame
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(230.0);
                            ui.set_min_height(118.0);
                            ui.label(RichText::new(preset.name()).size(17.0).strong());
                            ui.add_space(4.0);
                            ui.label(RichText::new(preset.blurb()).small());
                        });
                    })
                    .response
                    .interact(egui::Sense::click());
                if response.clicked() {
                    self.settings.apply(preset);
                }
            }
        });
        if self.settings.preset == Preset::Custom {
            ui.label(
                RichText::new("Custom: you changed a setting on the Performance page.").color(DIM),
            );
        }
        ui.add_space(14.0);
        ui.label(RichText::new(format!(
            "Upscaling: {}   Frame generation: {}",
            match self.settings.upscaler {
                Upscaler::Fsr => "FSR",
                Upscaler::DlssNative => "DLSS (native Streamline)",
                Upscaler::DlssOptiScaler => "DLSS (OptiScaler)",
            },
            match self.settings.framegen {
                FrameGen::Off => "off",
                FrameGen::OptiScaler => "OptiScaler 2x (End key)",
                FrameGen::SmoothMotion => "NVIDIA Smooth Motion",
                FrameGen::DlssStreamline => "DLSS Frame Generation 2x",
            }
        )));
        ui.add_space(16.0);
        let label = if self.game_running {
            "Game running..."
        } else {
            "PLAY"
        };
        let button = egui::Button::new(
            RichText::new(label)
                .size(24.0)
                .strong()
                .color(Color32::BLACK),
        )
        .fill(ACCENT);
        if ui
            .add_enabled(ok && !self.busy(), button.min_size(egui::vec2(260.0, 56.0)))
            .clicked()
        {
            self.start_game();
        }
        ui.label(
            RichText::new(
                "The first start translates every shader once (a few minutes, 6 workers). \
            You confirm the start screens and legal notices yourself.",
            )
            .color(DIM)
            .small(),
        );
    }

    fn graphics_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Upscaling & frame generation");
        ui.add_space(8.0);
        ui.label(RichText::new("Upscaler").strong());
        ui.radio_value(
            &mut self.settings.upscaler,
            Upscaler::Fsr,
            "AMD FSR (default)",
        );
        note(
            ui,
            "Requires a compatible local AMD FidelityFX upscaler DLL.",
        );
        ui.radio_value(
            &mut self.settings.upscaler,
            Upscaler::DlssOptiScaler,
            "NVIDIA DLSS (through OptiScaler)",
        );
        note(
            ui,
            "RTX cards. OptiScaler is downloaded and checked on first use; the DLSS library is taken from a game you \
            have installed (Steam, Epic, Xbox app). If none is found, FSR is used.",
        );
        ui.radio_value(&mut self.settings.upscaler, Upscaler::DlssNative, "NVIDIA DLSS (native Streamline)");
        if self.settings.upscaler == Upscaler::DlssNative {
            egui::ComboBox::from_label("DLSS quality")
                .selected_text(&self.settings.dlss_mode)
                .show_ui(ui, |ui| {
                    for (value, label) in [("ultraperformance", "Ultra Performance"), ("performance", "Performance"),
                        ("balanced", "Balanced"), ("quality", "Quality"), ("dlaa", "DLAA")]
                    {
                        ui.selectable_value(&mut self.settings.dlss_mode, value.to_owned(), label);
                    }
                });
            egui::ComboBox::from_label("DLSS model")
                .selected_text(&self.settings.dlss_model)
                .show_ui(ui, |ui| {
                    for (value, label) in [("default", "Default"), ("j", "J"), ("k", "K"), ("l", "L"), ("m", "M")] {
                        ui.selectable_value(&mut self.settings.dlss_model, value.to_owned(), label);
                    }
                });
        }
        ui.add_space(14.0);
        ui.label(RichText::new("Frame generation (off by default)").strong());
        note(
            ui,
            "Frame generation adds interpolated pictures between rendered gameplay frames. \
            Low render rates can produce visible artifacts during fast motion.",
        );
        ui.radio_value(&mut self.settings.framegen, FrameGen::Off, "Off");
        ui.radio_value(
            &mut self.settings.framegen,
            FrameGen::OptiScaler,
            "OptiScaler frame generation (2x, End key toggles it in game)",
        );
        if self.settings.framegen == FrameGen::OptiScaler {
            ui.indent("fg", |ui| {
                ui.checkbox(
                    &mut self.settings.framegen_at_start,
                    "On from the start (otherwise press End in game)",
                );
            });
        }
        ui.add_enabled_ui(self.smooth_motion_available, |ui| {
            ui.radio_value(
                &mut self.settings.framegen,
                FrameGen::SmoothMotion,
                "NVIDIA Smooth Motion (experimental driver feature)",
            );
        });
        if self.smooth_motion_available {
            note(
                ui,
                "Asks for administrator rights at start; switched off again when the game ends. Measured on an \
                RTX 4060 in the open world: no extra frames were shown, 12 instead of 19 pictures a second reached the \
                screen, much more stutter. Here for anyone who wants to try it with their own driver.",
            );
        } else {
            note(
                ui,
                "Needs a newer bin\\victory.exe (with the smooth-motion command).",
            );
        }
        ui.radio_value(
            &mut self.settings.framegen,
            FrameGen::DlssStreamline,
            "NVIDIA DLSS Frame Generation (Streamline, RTX 40/50)",
        );
        note(
            ui,
            "Uses the game's depth and motion vectors with native DLSS or FSR upscaling. NVIDIA Streamline is downloaded from NVIDIA's GitHub (276 MB, once) and checked on first use. Needs hardware-accelerated GPU scheduling on in Windows.",
        );
        if self.settings.framegen == FrameGen::DlssStreamline {
            ui.checkbox(&mut self.settings.dlss_fg_at_start, "Enable at launch");
            ui.add(egui::Slider::new(&mut self.settings.dlss_fg_multiplier, 2..=6).text("Requested multiplier"));
            note(ui, "F1 can switch a prepared backend live. The driver limits the supported multiplier.");
        }
        ui.add_space(10.0);
        ui.label(RichText::new("Multipliers above 2x: FSR frame generation and Smooth Motion are 2x only, and DLSS 3x/4x \
            needs an RTX 50 card.").color(DIM).small());
    }

    fn performance_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Performance");
        ui.label(RichText::new(format!("Preset: {}", self.settings.preset.name())).color(DIM));
        ui.add_space(8.0);
        ui.label(RichText::new("Render scale").strong());
        ui.radio_value(
            &mut self.settings.scale,
            Scale::Half,
            "Half (960x540 upscaled to 1080p): faster",
        );
        ui.radio_value(
            &mut self.settings.scale,
            Scale::TwoThirds,
            "The game's own 2/3: sharper, about 15% slower",
        );
        ui.add_space(10.0);
        ui.checkbox(&mut self.settings.menus_60, "Menus and videos at 60 fps");
        note(
            ui,
            "Smoother menus. The 3D world runs as fast as your PC allows either way.",
        );
        ui.add_space(10.0);
        ui.label(RichText::new("Speed-ups").strong());
        ui.checkbox(
            &mut self.settings.sync_in_place,
            "Faster descriptor sync: compare in place",
        );
        ui.checkbox(
            &mut self.settings.sync_fine,
            "Faster descriptor sync: smaller jobs",
        );
        note(
            ui,
            "Both together measured about 2% faster in the open world; no change to the picture.",
        );
        ui.add_space(10.0);
        ui.label(RichText::new("Worker threads").strong());
        ui.add(
            egui::Slider::new(&mut self.settings.draw_threads, 2..=16)
                .text("draw-call translation threads (tested best: 6)"),
        );
        ui.add(
            egui::Slider::new(&mut self.settings.landing_threads, 0..=16)
                .text("landing threads (0: automatic)"),
        );
        note(
            ui,
            "More is not faster here: 8 draw threads measured slower than 6 in the prison and the open world.",
        );
        ui.add_space(10.0);
        ui.label(RichText::new("Direct3D 12 Agility SDK").strong());
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.settings.agility,
                Agility::Auto,
                "Automatic (Windows 10 only)",
            );
            ui.radio_value(&mut self.settings.agility, Agility::On, "Always");
            ui.radio_value(&mut self.settings.agility, Agility::Off, "Never");
        });
        ui.add_space(10.0);
        ui.checkbox(
            &mut self.settings.raytracing,
            "Ray-traced shadows (experimental, much slower, uses the 2/3 scale)",
        );
        ui.add_space(10.0);
        egui::CollapsingHeader::new("Advanced: more environment variables (for testers)").show(
            ui,
            |ui| {
                note(ui, "One NAME=value a line, given to the game as they are.");
                ui.add(
                    egui::TextEdit::multiline(&mut self.settings.extra_env)
                        .desired_rows(4)
                        .desired_width(f32::INFINITY)
                        .code_editor(),
                );
            },
        );
        ui.add_space(10.0);
        if ui.button("Back to the Balanced preset").clicked() {
            self.settings.apply(Preset::Balanced);
        }
    }

    fn tools_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Tools");
        ui.add_space(8.0);
        let free = !self.busy();
        if ui
            .add_enabled(
                free && launch::is_game_dir(&self.settings.game_dir),
                egui::Button::new("Translate every shader again (6 workers)")
                    .min_size(egui::vec2(300.0, 32.0)),
            )
            .clicked()
        {
            self.tool_running = true;
            launch::warm(self.root.clone(), self.settings.clone(), self.send.clone());
        }
        note(
            ui,
            "Done by itself on the first start. Run it after changing ray tracing or advanced variables.",
        );
        ui.add_space(8.0);
        if ui
            .add_enabled(
                free,
                egui::Button::new("Import a save (folder or .zip)...")
                    .min_size(egui::vec2(300.0, 32.0)),
            )
            .clicked()
        {
            let picked = rfd::FileDialog::new()
                .set_title("A save .zip (Cancel: pick a folder instead)")
                .add_filter("zip", &["zip"])
                .pick_file()
                .or_else(|| {
                    rfd::FileDialog::new()
                        .set_title("A save folder")
                        .pick_folder()
                });
            if let Some(source) = picked {
                self.tool_running = true;
                launch::import_save(self.root.clone(), source, self.send.clone());
            }
        }
        note(
            ui,
            "Your own saves are backed up to save-backups\\ first. The game must be closed.",
        );
        ui.add_space(8.0);
        if ui
            .add_enabled(
                free,
                egui::Button::new("Check Resizable BAR").min_size(egui::vec2(300.0, 32.0)),
            )
            .clicked()
        {
            self.tool_running = true;
            launch::check_rebar(self.send.clone());
        }
        note(
            ui,
            "Resizable BAR on (in the BIOS) took the prison from 59.8 to 48.8 ms a frame here.",
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for (name, dir) in [
                ("Open logs", "logs"),
                ("Open saves", "data"),
                ("Open the install folder", ""),
            ] {
                if ui.button(name).clicked() {
                    let path = self.root.join(dir);
                    let _ = std::fs::create_dir_all(&path);
                    let _ = std::process::Command::new("explorer").arg(&path).spawn();
                }
            }
        });
    }

    fn log_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Messages").strong());
            if self.busy() {
                ui.spinner();
            }
            if ui.small_button("Clear").clicked() {
                self.log.clear();
            }
        });
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for line in &self.log {
                    ui.label(RichText::new(line).monospace().small());
                }
            });
    }
}

/// One line of what the settings are.
fn summary(s: &Settings) -> String {
    format!(
        "{} scale, {} draw threads{}{}{}",
        match s.scale {
            Scale::Half => "half",
            Scale::TwoThirds => "2/3",
        },
        s.draw_threads,
        if s.sync_in_place || s.sync_fine {
            ", fast sync"
        } else {
            ""
        },
        if s.menus_60 { ", 60 fps menus" } else { "" },
        if s.raytracing { ", ray tracing" } else { "" },
    )
}

fn note(ui: &mut egui::Ui, text: &str) {
    ui.indent(text, |ui| {
        ui.label(RichText::new(text).color(DIM).small());
    });
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        if self.busy() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("nav")
            .exact_size(200.0)
            .resizable(false)
            .show_inside(ui, |ui| self.sidebar(ui));
        egui::Panel::bottom("log")
            .exact_size(150.0)
            .resizable(false)
            .show_inside(ui, |ui| self.log_panel(ui));
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match self.page {
                    Page::Play => self.play_page(ui),
                    Page::Graphics => self.graphics_page(ui),
                    Page::Performance => self.performance_page(ui),
                    Page::Tools => self.tools_page(ui),
                });
        });
    }
}

fn initial_settings(root: &std::path::Path) -> Settings {
    if Settings::file(root).is_file() {
        return Settings::load(root);
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(folder) = exe.parent()
        && Settings::file(folder).is_file()
    {
        let settings = Settings::load(folder);
        let _ = std::fs::create_dir_all(root).and_then(|()| settings.save(root));
        return settings;
    }
    Settings::default()
}

fn main() -> eframe::Result {
    let root = payload::data_home().unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });
    if std::env::args_os().any(|arg| arg == "--verify") {
        let result = payload::install(&root).and_then(|()| launch::verify(&root));
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("VIctoryUnified")
            .with_inner_size([1000.0, 720.0])
            .with_min_inner_size([820.0, 560.0]),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "VIctoryUnified",
        options,
        Box::new(move |creation| {
            creation.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(App::new(root)))
        }),
    )
}
