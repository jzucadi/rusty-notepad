use chrono::Local;
use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::System;

use crate::constants::{DEFAULT_EDITOR_FONT_SIZE, SYSTEM_INFO_REFRESH_MS, WEATHER_REFRESH_SECS};
use crate::gpu;
use crate::theme;
use crate::weather::{self, WeatherInfo};

#[derive(Debug, Clone, PartialEq)]
pub enum PendingAction {
    New,
    Open,
    Exit,
}

pub struct NotepadApp {
    pub text: String,
    pub file_path: Option<PathBuf>,
    pub dirty: bool,
    pub show_unsaved_dialog: bool,
    pub pending_action: Option<PendingAction>,
    pub status_message: Option<String>,
    pub font_size: f32,
    pub dark_mode: bool,
    pub weather: Arc<Mutex<Option<WeatherInfo>>>,
    pub last_weather_fetch: Option<Instant>,
    pub system: System,
    pub cpu_usage: f32,
    pub gpu_usage: Option<f32>,
    pub last_system_refresh: Instant,
}

impl NotepadApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_mocha(&cc.egui_ctx);

        let weather = Arc::new(Mutex::new(None));

        // Fetch weather in background on startup
        let weather_clone = Arc::clone(&weather);
        thread::spawn(move || {
            if let Some(info) = weather::fetch_weather() {
                if let Ok(mut w) = weather_clone.lock() {
                    *w = Some(info);
                }
            }
        });

        let mut system = System::new_all();
        system.refresh_cpu_all();

        Self {
            text: String::new(),
            file_path: None,
            dirty: false,
            show_unsaved_dialog: false,
            pending_action: None,
            status_message: None,
            font_size: DEFAULT_EDITOR_FONT_SIZE,
            dark_mode: true,
            weather,
            last_weather_fetch: Some(Instant::now()),
            system,
            cpu_usage: 0.0,
            gpu_usage: None,
            last_system_refresh: Instant::now(),
        }
    }

    pub fn refresh_weather_if_needed(&mut self) {
        let should_refresh = self.last_weather_fetch
            .map(|t| t.elapsed() > Duration::from_secs(WEATHER_REFRESH_SECS))
            .unwrap_or(true);

        if should_refresh {
            self.last_weather_fetch = Some(Instant::now());
            let weather_clone = Arc::clone(&self.weather);
            thread::spawn(move || {
                if let Some(info) = weather::fetch_weather() {
                    if let Ok(mut w) = weather_clone.lock() {
                        *w = Some(info);
                    }
                }
            });
        }
    }

    pub fn refresh_system_info(&mut self) {
        if self.last_system_refresh.elapsed() > Duration::from_millis(SYSTEM_INFO_REFRESH_MS) {
            self.system.refresh_cpu_all();
            self.cpu_usage = self.system.global_cpu_usage();
            self.gpu_usage = gpu::get_gpu_usage();
            self.last_system_refresh = Instant::now();
        }
    }

    pub fn apply_theme(&self, ctx: &egui::Context) {
        if self.dark_mode {
            theme::apply_mocha(ctx);
        } else {
            theme::apply_latte(ctx);
        }
    }

    pub fn window_title(&self) -> String {
        Local::now().format("%A, %B %d, %Y  %I:%M:%S %p").to_string()
    }

    pub fn new_file(&mut self) {
        if self.dirty {
            self.show_unsaved_dialog = true;
            self.pending_action = Some(PendingAction::New);
        } else {
            self.do_new_file();
        }
    }

    pub fn do_new_file(&mut self) {
        self.text.clear();
        self.file_path = None;
        self.dirty = false;
        self.status_message = Some("New file created".to_string());
    }

    pub fn open_file(&mut self) {
        if self.dirty {
            self.show_unsaved_dialog = true;
            self.pending_action = Some(PendingAction::Open);
        } else {
            self.do_open_file();
        }
    }

    pub fn do_open_file(&mut self) {
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

    pub fn save_file(&mut self) {
        if let Some(ref path) = self.file_path {
            self.write_file(path.clone());
        } else {
            self.save_file_as();
        }
    }

    pub fn save_file_as(&mut self) {
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

    pub fn request_exit(&mut self, ctx: &egui::Context) {
        if self.dirty {
            self.show_unsaved_dialog = true;
            self.pending_action = Some(PendingAction::Exit);
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    pub fn execute_pending_action(&mut self, action: &PendingAction, ctx: &egui::Context) {
        match action {
            PendingAction::New => self.do_new_file(),
            PendingAction::Open => self.do_open_file(),
            PendingAction::Exit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }

    pub fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        let ctrl = ctx.input(|i| i.modifiers.ctrl || i.modifiers.mac_cmd);
        let shift = ctx.input(|i| i.modifiers.shift);

        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::N)) {
            self.new_file();
        }

        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::O)) {
            self.open_file();
        }

        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::S)) {
            if shift {
                self.save_file_as();
            } else {
                self.save_file();
            }
        }
    }

    pub fn handle_close_request(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.dirty && !self.show_unsaved_dialog {
                self.show_unsaved_dialog = true;
                self.pending_action = Some(PendingAction::Exit);
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            } else if self.show_unsaved_dialog {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            }
        }
    }
}
