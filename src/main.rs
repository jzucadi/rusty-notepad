mod app;
mod constants;
mod gpu;
mod theme;
mod ui;
mod weather;

use eframe::egui;
use std::time::Duration;

use app::NotepadApp;
use constants::{MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH, REPAINT_INTERVAL_SECS, WINDOW_HEIGHT, WINDOW_WIDTH};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_min_inner_size([MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT])
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

impl eframe::App for NotepadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(REPAINT_INTERVAL_SECS));

        self.refresh_weather_if_needed();
        self.refresh_system_info();
        self.handle_close_request(ctx);
        self.handle_keyboard_shortcuts(ctx);
        self.handle_unsaved_dialog(ctx);

        self.render_title_bar(ctx);
        self.render_menu_bar(ctx);
        self.render_status_bar(ctx);
        self.render_text_editor(ctx);
    }
}
