use eframe::egui;

use crate::app::NotepadApp;
use crate::constants::{
    ELEMENT_SPACING, FONT_SIZE_EXTRA_LARGE, FONT_SIZE_LARGE, FONT_SIZE_MEDIUM, FONT_SIZE_SMALL,
    FONT_SIZE_STEP, MAX_FONT_SIZE, MIN_FONT_SIZE, STATUS_BAR_FONT_SIZE, STATUS_BAR_MARGIN_H,
    STATUS_BAR_MARGIN_V, THEME_ICON_SIZE, TITLE_BAR_FONT_SIZE, TITLE_BAR_HEIGHT,
    TITLE_CENTER_WIDTH, TRAFFIC_LIGHTS_SPACE, WEATHER_SPACING,
};
use crate::theme;

impl NotepadApp {
    pub fn render_title_bar(&self, ctx: &egui::Context) {
        let (base_color, text_color) = theme::get_theme_colors(self.dark_mode);

        let weather_text = if let Ok(weather) = self.weather.lock() {
            if let Some(ref info) = *weather {
                format!("{} {:.0}°F {}", info.icon, info.temperature_f, info.description)
            } else {
                "Loading...".to_string()
            }
        } else {
            String::new()
        };

        egui::TopBottomPanel::top("title_bar")
            .exact_height(TITLE_BAR_HEIGHT)
            .frame(egui::Frame::none().fill(base_color))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(TRAFFIC_LIGHTS_SPACE);

                    let time_text = self.window_title();
                    let available_width = ui.available_width();
                    ui.add_space((available_width - TITLE_CENTER_WIDTH) / 2.0);
                    ui.label(egui::RichText::new(time_text).color(text_color).size(TITLE_BAR_FONT_SIZE));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(WEATHER_SPACING);
                        ui.label(egui::RichText::new(&weather_text).color(text_color).size(TITLE_BAR_FONT_SIZE));
                    });
                });
            });
    }

    pub fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("\u{1F4C4} File", |ui| {
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

                ui.menu_button("\u{2699} Settings", |ui| {
                    ui.label("Editor Font Size");
                    ui.horizontal(|ui| {
                        if ui.button("-").clicked() {
                            self.font_size = (self.font_size - FONT_SIZE_STEP).max(MIN_FONT_SIZE);
                        }
                        ui.label(format!("{:.0}", self.font_size));
                        if ui.button("+").clicked() {
                            self.font_size = (self.font_size + FONT_SIZE_STEP).min(MAX_FONT_SIZE);
                        }
                    });

                    ui.separator();

                    ui.menu_button("Presets", |ui| {
                        if ui.button("Small (12)").clicked() {
                            self.font_size = FONT_SIZE_SMALL;
                            ui.close_menu();
                        }
                        if ui.button("Medium (14)").clicked() {
                            self.font_size = FONT_SIZE_MEDIUM;
                            ui.close_menu();
                        }
                        if ui.button("Large (18)").clicked() {
                            self.font_size = FONT_SIZE_LARGE;
                            ui.close_menu();
                        }
                        if ui.button("Extra Large (24)").clicked() {
                            self.font_size = FONT_SIZE_EXTRA_LARGE;
                            ui.close_menu();
                        }
                    });
                });
            });
        });
    }

    pub fn render_status_bar(&mut self, ctx: &egui::Context) {
        let (base_color, _) = theme::get_theme_colors(self.dark_mode);

        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::none().fill(base_color).inner_margin(egui::Margin::symmetric(STATUS_BAR_MARGIN_H, STATUS_BAR_MARGIN_V)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (theme_icon, icon_color) = if self.dark_mode {
                        ("\u{1F319}", egui::Color32::from_rgb(249, 226, 175))
                    } else {
                        ("\u{2600}", egui::Color32::from_rgb(223, 142, 29))
                    };

                    let button = egui::Button::new(egui::RichText::new(theme_icon).color(icon_color).size(THEME_ICON_SIZE))
                        .frame(false);
                    if ui.add(button).clicked() {
                        self.dark_mode = !self.dark_mode;
                        self.apply_theme(ctx);
                    }

                    if let Some(ref msg) = self.status_message {
                        ui.add_space(ELEMENT_SPACING);
                        ui.label(msg);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let stats = &self.system_stats;
                        let gpu_text = stats.gpu_usage
                            .map(|u| format!("{:.1}%", u))
                            .unwrap_or_else(|| "N/A".to_string());
                        let temp_text = stats.cpu_temp
                            .map(|t| format!("{:.0}°C", t))
                            .unwrap_or_else(|| "N/A".to_string());

                        ui.label(egui::RichText::new(format!(
                            "CPU: {:.1}% | GPU: {} | RAM: {:.1}% | Temp: {}",
                            stats.cpu_usage, gpu_text, stats.ram_usage, temp_text
                        )).size(STATUS_BAR_FONT_SIZE));
                    });
                });
            });
    }

    pub fn render_text_editor(&mut self, ctx: &egui::Context) {
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

    pub fn handle_unsaved_dialog(&mut self, ctx: &egui::Context) {
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
                ui.add_space(ELEMENT_SPACING);

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.save_file();
                        if !self.dirty {
                            close_dialog = true;
                            if let Some(ref action) = pending {
                                self.execute_pending_action(action, ctx);
                            }
                        }
                    }

                    if ui.button("Don't Save").clicked() {
                        close_dialog = true;
                        if let Some(ref action) = pending {
                            self.execute_pending_action(action, ctx);
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
}
