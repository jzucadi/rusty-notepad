use chrono::Local;
use eframe::egui;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false),
        ..Default::default()
    };

    eframe::run_native(
        "Rusty Notepad",
        options,
        Box::new(|cc| Ok(Box::new(NotepadApp::new(cc)))),
    )
}

// IP Geolocation response
#[derive(Debug, Deserialize)]
struct GeoResponse {
    lat: f64,
    lon: f64,
}

struct NotepadApp {
    text: String,
    file_path: Option<PathBuf>,
    dirty: bool,
    show_unsaved_dialog: bool,
    pending_action: Option<PendingAction>,
    status_message: Option<String>,
    font_size: f32,
    dark_mode: bool,
    weather: Arc<Mutex<Option<WeatherInfo>>>,
    last_weather_fetch: Option<Instant>,
}

#[derive(Clone)]
struct WeatherInfo {
    temperature_f: f64,
    description: String,
    icon: String,
}

#[derive(Clone)]
enum PendingAction {
    New,
    Open,
    Exit,
}

impl NotepadApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Apply Catppuccin Mocha theme manually
        Self::apply_catppuccin_mocha(&cc.egui_ctx);

        let weather = Arc::new(Mutex::new(None));

        // Fetch weather in background on startup
        let weather_clone = Arc::clone(&weather);
        thread::spawn(move || {
            if let Some(info) = Self::fetch_weather() {
                if let Ok(mut w) = weather_clone.lock() {
                    *w = Some(info);
                }
            }
        });

        Self {
            text: String::new(),
            file_path: None,
            dirty: false,
            show_unsaved_dialog: false,
            pending_action: None,
            status_message: None,
            font_size: 14.0,
            dark_mode: true,
            weather,
            last_weather_fetch: Some(Instant::now()),
        }
    }

    fn fetch_weather() -> Option<WeatherInfo> {
        // Use ip-api.com to get location (no API key needed)
        let geo_url = "http://ip-api.com/json/";
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .ok()?;

        let geo_resp: GeoResponse = client.get(geo_url).send().ok()?.json().ok()?;

        // Use Open-Meteo API (free, no API key needed)
        let weather_url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true&temperature_unit=fahrenheit",
            geo_resp.lat, geo_resp.lon
        );

        let resp = client.get(&weather_url).send().ok()?;
        let json: serde_json::Value = resp.json().ok()?;

        let current = json.get("current_weather")?;
        let temp = current.get("temperature")?.as_f64()?;
        let weather_code = current.get("weathercode")?.as_i64().unwrap_or(0);

        // Convert weather code to description and icon
        let (description, icon) = match weather_code {
            0 => ("Clear", "\u{2600}"),              // ☀ sun
            1 | 2 | 3 => ("Partly cloudy", "\u{26C5}"), // ⛅ sun behind cloud
            45 | 48 => ("Foggy", "\u{1F32B}"),       // 🌫 fog
            51 | 53 | 55 => ("Drizzle", "\u{1F327}"), // 🌧 cloud with rain
            61 | 63 | 65 => ("Rain", "\u{1F327}"),   // 🌧 cloud with rain
            71 | 73 | 75 => ("Snow", "\u{2744}"),    // ❄ snowflake
            77 => ("Snow grains", "\u{2744}"),       // ❄ snowflake
            80 | 81 | 82 => ("Showers", "\u{1F327}"), // 🌧 cloud with rain
            85 | 86 => ("Snow showers", "\u{1F328}"), // 🌨 cloud with snow
            95 => ("Thunderstorm", "\u{26C8}"),      // ⛈ thunder cloud and rain
            96 | 99 => ("Thunderstorm", "\u{26C8}"), // ⛈ thunder cloud and rain
            _ => ("Unknown", "\u{2601}"),            // ☁ cloud
        };

        Some(WeatherInfo {
            temperature_f: temp,
            description: description.to_string(),
            icon: icon.to_string(),
        })
    }

    fn refresh_weather_if_needed(&mut self) {
        // Refresh weather every 10 minutes
        let should_refresh = self.last_weather_fetch
            .map(|t| t.elapsed() > Duration::from_secs(600))
            .unwrap_or(true);

        if should_refresh {
            self.last_weather_fetch = Some(Instant::now());
            let weather_clone = Arc::clone(&self.weather);
            thread::spawn(move || {
                if let Some(info) = Self::fetch_weather() {
                    if let Ok(mut w) = weather_clone.lock() {
                        *w = Some(info);
                    }
                }
            });
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        if self.dark_mode {
            Self::apply_catppuccin_mocha(ctx);
        } else {
            Self::apply_catppuccin_latte(ctx);
        }
    }

    fn apply_catppuccin_latte(ctx: &egui::Context) {
        // Catppuccin Latte colors (light theme)
        let base = egui::Color32::from_rgb(239, 241, 245);
        let mantle = egui::Color32::from_rgb(230, 233, 239);
        let crust = egui::Color32::from_rgb(220, 224, 232);
        let _text = egui::Color32::from_rgb(76, 79, 105);
        let _subtext0 = egui::Color32::from_rgb(108, 111, 133);
        let surface0 = egui::Color32::from_rgb(204, 208, 218);
        let surface1 = egui::Color32::from_rgb(188, 192, 204);
        let surface2 = egui::Color32::from_rgb(172, 176, 190);
        let blue = egui::Color32::from_rgb(30, 102, 245);
        let sapphire = egui::Color32::from_rgb(32, 159, 181);

        let mut style = (*ctx.style()).clone();

        // Window
        style.visuals.window_fill = base;
        style.visuals.panel_fill = base;
        style.visuals.faint_bg_color = mantle;
        style.visuals.extreme_bg_color = crust;

        // Text - use #4c4f69 for UI text in light mode
        let ui_text = egui::Color32::from_rgb(76, 79, 105); // #4c4f69
        style.visuals.override_text_color = Some(ui_text);

        // Widgets - use base for borders/strokes, ui_text for text
        style.visuals.widgets.noninteractive.bg_fill = surface0;
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, ui_text);
        style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, base);

        style.visuals.widgets.inactive.bg_fill = surface0;
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, ui_text);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, base);

        style.visuals.widgets.hovered.bg_fill = surface1;
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, ui_text);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, base);

        style.visuals.widgets.active.bg_fill = surface2;
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, ui_text);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, base);

        style.visuals.widgets.open.bg_fill = surface1;
        style.visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, ui_text);
        style.visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, base);

        // Selection
        style.visuals.selection.bg_fill = blue.gamma_multiply(0.3);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, base);

        // Hyperlinks
        style.visuals.hyperlink_color = sapphire;

        // Window stroke - use base
        style.visuals.window_stroke = egui::Stroke::new(1.0, base);

        // Light mode
        style.visuals.dark_mode = false;

        // Set fixed font size of 16 for all UI elements
        let ui_font_size = 16.0;
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(ui_font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(ui_font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(ui_font_size * 1.2, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(ui_font_size * 0.85, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(ui_font_size, egui::FontFamily::Monospace),
        );

        ctx.set_style(style);
    }

    fn apply_catppuccin_mocha(ctx: &egui::Context) {
        // Catppuccin Mocha colors
        let base = egui::Color32::from_rgb(30, 30, 46);
        let mantle = egui::Color32::from_rgb(24, 24, 37);
        let crust = egui::Color32::from_rgb(17, 17, 27);
        let _text = egui::Color32::from_rgb(205, 214, 244);
        let _subtext0 = egui::Color32::from_rgb(166, 173, 200);
        let surface0 = egui::Color32::from_rgb(49, 50, 68);
        let surface1 = egui::Color32::from_rgb(69, 71, 90);
        let surface2 = egui::Color32::from_rgb(88, 91, 112);
        let blue = egui::Color32::from_rgb(137, 180, 250);
        let sapphire = egui::Color32::from_rgb(116, 199, 236);
        let ui_text = egui::Color32::from_rgb(138, 173, 244); // #8aadf4

        let mut style = (*ctx.style()).clone();

        // Window
        style.visuals.window_fill = base;
        style.visuals.panel_fill = base;
        style.visuals.faint_bg_color = mantle;
        style.visuals.extreme_bg_color = crust;

        // Text - use #8aadf4 for UI text in dark mode
        style.visuals.override_text_color = Some(ui_text);

        // Widgets - use base for borders/strokes, ui_text for text
        style.visuals.widgets.noninteractive.bg_fill = surface0;
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, ui_text);
        style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, base);

        style.visuals.widgets.inactive.bg_fill = surface0;
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, ui_text);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, base);

        style.visuals.widgets.hovered.bg_fill = surface1;
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, ui_text);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, base);

        style.visuals.widgets.active.bg_fill = surface2;
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, ui_text);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, base);

        style.visuals.widgets.open.bg_fill = surface1;
        style.visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, ui_text);
        style.visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, base);

        // Selection
        style.visuals.selection.bg_fill = blue.gamma_multiply(0.4);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, base);

        // Hyperlinks
        style.visuals.hyperlink_color = sapphire;

        // Window stroke - use base
        style.visuals.window_stroke = egui::Stroke::new(1.0, base);

        // Dark mode
        style.visuals.dark_mode = true;

        // Set fixed font size of 16 for all UI elements
        let ui_font_size = 16.0;
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(ui_font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(ui_font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(ui_font_size * 1.2, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(ui_font_size * 0.85, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(ui_font_size, egui::FontFamily::Monospace),
        );

        ctx.set_style(style);
    }

    fn window_title(&self) -> String {
        Local::now().format("%A, %B %d, %Y  %I:%M:%S %p").to_string()
    }

    fn new_file(&mut self) {
        if self.dirty {
            self.show_unsaved_dialog = true;
            self.pending_action = Some(PendingAction::New);
        } else {
            self.do_new_file();
        }
    }

    fn do_new_file(&mut self) {
        self.text.clear();
        self.file_path = None;
        self.dirty = false;
        self.status_message = Some("New file created".to_string());
    }

    fn open_file(&mut self) {
        if self.dirty {
            self.show_unsaved_dialog = true;
            self.pending_action = Some(PendingAction::Open);
        } else {
            self.do_open_file();
        }
    }

    fn do_open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            match fs::read_to_string(&path) {
                Ok(contents) => {
                    self.text = contents;
                    self.file_path = Some(path.clone());
                    self.dirty = false;
                    self.status_message = Some(format!("Opened: {}", path.display()));
                }
                Err(e) => {
                    self.status_message = Some(format!("Error opening file: {}", e));
                }
            }
        }
    }

    fn save_file(&mut self) {
        if let Some(ref path) = self.file_path {
            self.write_file(path.clone());
        } else {
            self.save_file_as();
        }
    }

    fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt"])
            .add_filter("All files", &["*"])
            .save_file()
        {
            self.write_file(path);
        }
    }

    fn write_file(&mut self, path: PathBuf) {
        match fs::write(&path, &self.text) {
            Ok(_) => {
                self.file_path = Some(path.clone());
                self.dirty = false;
                self.status_message = Some(format!("Saved: {}", path.display()));
            }
            Err(e) => {
                self.status_message = Some(format!("Error saving file: {}", e));
            }
        }
    }

    fn request_exit(&mut self, ctx: &egui::Context) {
        if self.dirty {
            self.show_unsaved_dialog = true;
            self.pending_action = Some(PendingAction::Exit);
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn handle_unsaved_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_unsaved_dialog {
            return;
        }

        let mut close_dialog = false;
        let pending = self.pending_action.clone();

        egui::Window::new("Unsaved Changes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("You have unsaved changes. What would you like to do?");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.save_file();
                        if !self.dirty {
                            // Save succeeded
                            close_dialog = true;
                            if let Some(action) = &pending {
                                match action {
                                    PendingAction::New => self.do_new_file(),
                                    PendingAction::Open => self.do_open_file(),
                                    PendingAction::Exit => {
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                    }
                                }
                            }
                        }
                    }

                    if ui.button("Don't Save").clicked() {
                        close_dialog = true;
                        if let Some(action) = &pending {
                            match action {
                                PendingAction::New => self.do_new_file(),
                                PendingAction::Open => self.do_open_file(),
                                PendingAction::Exit => {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        close_dialog = true;
                    }
                });
            });

        if close_dialog {
            self.show_unsaved_dialog = false;
            self.pending_action = None;
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        let ctrl = ctx.input(|i| i.modifiers.ctrl || i.modifiers.mac_cmd);
        let shift = ctx.input(|i| i.modifiers.shift);

        // Ctrl+N: New
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::N)) {
            self.new_file();
        }

        // Ctrl+O: Open
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::O)) {
            self.open_file();
        }

        // Ctrl+S: Save, Ctrl+Shift+S: Save As
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::S)) {
            if shift {
                self.save_file_as();
            } else {
                self.save_file();
            }
        }
    }
}

impl eframe::App for NotepadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint every second to update the time in the title bar
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        // Refresh weather if needed (every 10 minutes)
        self.refresh_weather_if_needed();

        // Handle close request - check if window close was requested
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.dirty && !self.show_unsaved_dialog {
                self.show_unsaved_dialog = true;
                self.pending_action = Some(PendingAction::Exit);
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            } else if self.show_unsaved_dialog {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            }
        }

        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx);

        // Handle unsaved changes dialog
        self.handle_unsaved_dialog(ctx);

        // Custom title bar with theme-aware colors
        let title_bar_height = 32.0;
        let (base_color, text_color) = if self.dark_mode {
            (
                egui::Color32::from_rgb(30, 30, 46),    // Mocha base
                egui::Color32::from_rgb(138, 173, 244), // #8aadf4
            )
        } else {
            (
                egui::Color32::from_rgb(239, 241, 245), // Latte base
                egui::Color32::from_rgb(76, 79, 105),   // #4c4f69
            )
        };

        // Get weather info
        let weather_text = if let Ok(weather) = self.weather.lock() {
            if let Some(ref info) = *weather {
                format!("{} {:.0}°F {}", info.icon, info.temperature_f, info.description)
            } else {
                "Loading...".to_string()
            }
        } else {
            "".to_string()
        };

        egui::TopBottomPanel::top("title_bar")
            .exact_height(title_bar_height)
            .frame(egui::Frame::none().fill(base_color))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // Left side - space for traffic lights
                    ui.add_space(80.0);

                    // Center - date and time
                    let time_text = self.window_title();
                    let available_width = ui.available_width();
                    ui.add_space((available_width - 300.0) / 2.0);
                    ui.label(egui::RichText::new(time_text).color(text_color).size(14.0));

                    // Right side - weather
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(15.0);
                        ui.label(egui::RichText::new(&weather_text).color(text_color).size(14.0));
                    });
                });
            });

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("\u{1F4C4} File", |ui| {  // 📄 document icon
                    if ui
                        .add(egui::Button::new("New").shortcut_text("Ctrl+N"))
                        .clicked()
                    {
                        self.new_file();
                        ui.close_menu();
                    }

                    if ui
                        .add(egui::Button::new("Open...").shortcut_text("Ctrl+O"))
                        .clicked()
                    {
                        self.open_file();
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui
                        .add(egui::Button::new("Save").shortcut_text("Ctrl+S"))
                        .clicked()
                    {
                        self.save_file();
                        ui.close_menu();
                    }

                    if ui
                        .add(egui::Button::new("Save As...").shortcut_text("Ctrl+Shift+S"))
                        .clicked()
                    {
                        self.save_file_as();
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Exit").clicked() {
                        self.request_exit(ctx);
                        ui.close_menu();
                    }
                });

                ui.menu_button("\u{2699} Settings", |ui| {  // ⚙ gear icon
                    ui.label("Editor Font Size");
                    ui.horizontal(|ui| {
                        if ui.button("-").clicked() {
                            self.font_size = (self.font_size - 1.0).max(8.0);
                        }
                        ui.label(format!("{:.0}", self.font_size));
                        if ui.button("+").clicked() {
                            self.font_size = (self.font_size + 1.0).min(48.0);
                        }
                    });

                    ui.separator();

                    ui.menu_button("Presets", |ui| {
                        if ui.button("Small (12)").clicked() {
                            self.font_size = 12.0;
                            ui.close_menu();
                        }
                        if ui.button("Medium (14)").clicked() {
                            self.font_size = 14.0;
                            ui.close_menu();
                        }
                        if ui.button("Large (18)").clicked() {
                            self.font_size = 18.0;
                            ui.close_menu();
                        }
                        if ui.button("Extra Large (24)").clicked() {
                            self.font_size = 24.0;
                            ui.close_menu();
                        }
                    });
                });
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::none().fill(base_color).inner_margin(egui::Margin::symmetric(8.0, 4.0)))
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Theme toggle on the left (sun for light, moon for dark)
                // Show moon in dark mode (click to go light), sun in light mode (click to go dark)
                let (theme_icon, icon_color) = if self.dark_mode {
                    ("\u{1F319}", egui::Color32::from_rgb(249, 226, 175)) // 🌙 crescent moon, yellow/gold
                } else {
                    ("\u{2600}", egui::Color32::from_rgb(223, 142, 29))   // ☀ sun, orange
                };

                let button = egui::Button::new(egui::RichText::new(theme_icon).color(icon_color).size(18.0))
                    .frame(false);
                if ui.add(button).clicked() {
                    self.dark_mode = !self.dark_mode;
                    self.apply_theme(ctx);
                }

                if let Some(ref msg) = self.status_message {
                    ui.add_space(10.0);
                    ui.label(msg);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let lines = self.text.lines().count().max(1);
                    let chars = self.text.len();
                    ui.label(format!("Lines: {} | Chars: {}", lines, chars));
                });
            });
        });

        // Main text editor
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let editor_font = egui::FontId::new(self.font_size, egui::FontFamily::Monospace);
                    let response = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut self.text)
                            .font(editor_font)
                            .desired_width(f32::INFINITY),
                    );

                    if response.changed() {
                        self.dirty = true;
                    }
                });
        });
    }

}
