use chrono::Local;
use eframe::egui;
use std::fs;
use std::path::PathBuf;

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

struct NotepadApp {
    text: String,
    file_path: Option<PathBuf>,
    dirty: bool,
    show_unsaved_dialog: bool,
    pending_action: Option<PendingAction>,
    status_message: Option<String>,
    font_size: f32,
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

        Self {
            text: String::new(),
            file_path: None,
            dirty: false,
            show_unsaved_dialog: false,
            pending_action: None,
            status_message: None,
            font_size: 14.0,
        }
    }

    fn apply_catppuccin_mocha(ctx: &egui::Context) {
        // Catppuccin Mocha colors
        let base = egui::Color32::from_rgb(30, 30, 46);
        let mantle = egui::Color32::from_rgb(24, 24, 37);
        let crust = egui::Color32::from_rgb(17, 17, 27);
        let text = egui::Color32::from_rgb(205, 214, 244);
        let subtext0 = egui::Color32::from_rgb(166, 173, 200);
        let surface0 = egui::Color32::from_rgb(49, 50, 68);
        let surface1 = egui::Color32::from_rgb(69, 71, 90);
        let surface2 = egui::Color32::from_rgb(88, 91, 112);
        let overlay0 = egui::Color32::from_rgb(108, 112, 134);
        let blue = egui::Color32::from_rgb(137, 180, 250);
        let lavender = egui::Color32::from_rgb(180, 190, 254);
        let sapphire = egui::Color32::from_rgb(116, 199, 236);

        let mut style = (*ctx.style()).clone();

        // Window
        style.visuals.window_fill = base;
        style.visuals.panel_fill = base;
        style.visuals.faint_bg_color = mantle;
        style.visuals.extreme_bg_color = crust;

        // Text
        style.visuals.override_text_color = Some(text);

        // Widgets
        style.visuals.widgets.noninteractive.bg_fill = surface0;
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, subtext0);
        style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, surface1);

        style.visuals.widgets.inactive.bg_fill = surface0;
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, surface1);

        style.visuals.widgets.hovered.bg_fill = surface1;
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, text);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, lavender);

        style.visuals.widgets.active.bg_fill = surface2;
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, text);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, blue);

        style.visuals.widgets.open.bg_fill = surface1;
        style.visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, text);
        style.visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, surface2);

        // Selection
        style.visuals.selection.bg_fill = blue.gamma_multiply(0.4);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, lavender);

        // Hyperlinks
        style.visuals.hyperlink_color = sapphire;

        // Window stroke
        style.visuals.window_stroke = egui::Stroke::new(1.0, overlay0);

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

        // Custom title bar with Catppuccin Mocha colors
        let title_bar_height = 32.0;
        let mantle = egui::Color32::from_rgb(24, 24, 37);
        let text_color = egui::Color32::from_rgb(205, 214, 244);

        egui::TopBottomPanel::top("title_bar")
            .exact_height(title_bar_height)
            .frame(egui::Frame::none().fill(mantle))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // Leave space for native macOS traffic light buttons
                    ui.add_space(80.0);

                    // Center the date/time in the title bar
                    let time_text = self.window_title();
                    let available = ui.available_width();
                    let text_width = ui.fonts(|f| {
                        f.glyph_width(&egui::FontId::default(), ' ') * time_text.len() as f32
                    });
                    ui.add_space((available - text_width) / 2.0 - 40.0);

                    ui.label(egui::RichText::new(time_text).color(text_color).size(14.0));
                });
            });

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
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

                ui.menu_button("Settings", |ui| {
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
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(ref msg) = self.status_message {
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
