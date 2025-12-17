use eframe::egui;

use crate::constants::{HEADING_SCALE, SMALL_TEXT_SCALE, UI_FONT_SIZE};

pub struct CatppuccinPalette {
    pub base: egui::Color32,
    pub mantle: egui::Color32,
    pub crust: egui::Color32,
    pub surface0: egui::Color32,
    pub surface1: egui::Color32,
    pub surface2: egui::Color32,
    pub blue: egui::Color32,
    pub sapphire: egui::Color32,
    pub text: egui::Color32,
    pub selection_alpha: f32,
    pub is_dark: bool,
}

impl CatppuccinPalette {
    pub fn latte() -> Self {
        Self {
            base: egui::Color32::from_rgb(239, 241, 245),
            mantle: egui::Color32::from_rgb(230, 233, 239),
            crust: egui::Color32::from_rgb(220, 224, 232),
            surface0: egui::Color32::from_rgb(204, 208, 218),
            surface1: egui::Color32::from_rgb(188, 192, 204),
            surface2: egui::Color32::from_rgb(172, 176, 190),
            blue: egui::Color32::from_rgb(30, 102, 245),
            sapphire: egui::Color32::from_rgb(32, 159, 181),
            text: egui::Color32::from_rgb(76, 79, 105),
            selection_alpha: 0.3,
            is_dark: false,
        }
    }

    pub fn mocha() -> Self {
        Self {
            base: egui::Color32::from_rgb(30, 30, 46),
            mantle: egui::Color32::from_rgb(24, 24, 37),
            crust: egui::Color32::from_rgb(17, 17, 27),
            surface0: egui::Color32::from_rgb(49, 50, 68),
            surface1: egui::Color32::from_rgb(69, 71, 90),
            surface2: egui::Color32::from_rgb(88, 91, 112),
            blue: egui::Color32::from_rgb(137, 180, 250),
            sapphire: egui::Color32::from_rgb(116, 199, 236),
            text: egui::Color32::from_rgb(138, 173, 244),
            selection_alpha: 0.4,
            is_dark: true,
        }
    }
}

pub fn apply_latte(ctx: &egui::Context) {
    apply_palette(ctx, &CatppuccinPalette::latte());
}

pub fn apply_mocha(ctx: &egui::Context) {
    apply_palette(ctx, &CatppuccinPalette::mocha());
}

pub fn apply_palette(ctx: &egui::Context, palette: &CatppuccinPalette) {
    let mut style = (*ctx.style()).clone();

    // Window
    style.visuals.window_fill = palette.base;
    style.visuals.panel_fill = palette.base;
    style.visuals.faint_bg_color = palette.mantle;
    style.visuals.extreme_bg_color = palette.crust;

    // Text
    style.visuals.override_text_color = Some(palette.text);

    // Widgets
    style.visuals.widgets.noninteractive.bg_fill = palette.surface0;
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, palette.text);
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, palette.base);

    style.visuals.widgets.inactive.bg_fill = palette.surface0;
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, palette.text);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, palette.base);

    style.visuals.widgets.hovered.bg_fill = palette.surface1;
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, palette.text);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, palette.base);

    style.visuals.widgets.active.bg_fill = palette.surface2;
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, palette.text);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, palette.base);

    style.visuals.widgets.open.bg_fill = palette.surface1;
    style.visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, palette.text);
    style.visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, palette.base);

    // Selection
    style.visuals.selection.bg_fill = palette.blue.gamma_multiply(palette.selection_alpha);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, palette.base);

    // Hyperlinks
    style.visuals.hyperlink_color = palette.sapphire;

    // Window stroke
    style.visuals.window_stroke = egui::Stroke::new(1.0, palette.base);

    // Dark mode flag
    style.visuals.dark_mode = palette.is_dark;

    // Set font sizes for all UI elements
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(UI_FONT_SIZE, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(UI_FONT_SIZE, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(UI_FONT_SIZE * HEADING_SCALE, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(UI_FONT_SIZE * SMALL_TEXT_SCALE, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(UI_FONT_SIZE, egui::FontFamily::Monospace),
    );

    ctx.set_style(style);
}

pub fn get_theme_colors(dark_mode: bool) -> (egui::Color32, egui::Color32) {
    if dark_mode {
        (
            egui::Color32::from_rgb(30, 30, 46),    // Mocha base
            egui::Color32::from_rgb(138, 173, 244), // Mocha text
        )
    } else {
        (
            egui::Color32::from_rgb(239, 241, 245), // Latte base
            egui::Color32::from_rgb(76, 79, 105),   // Latte text
        )
    }
}
