use nih_plug_egui::egui;

// ── Background colors ──
pub const BG_DARK: egui::Color32 = egui::Color32::from_rgb(0x0d, 0x11, 0x17);
pub const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(0x16, 0x1b, 0x22);
pub const BG_SURFACE: egui::Color32 = egui::Color32::from_rgb(0x21, 0x26, 0x2d);
pub const BG_ELEVATED: egui::Color32 = egui::Color32::from_rgb(0x2d, 0x33, 0x3b);

// ── Fretboard colors ──
pub const FRETBOARD_WOOD: egui::Color32 = egui::Color32::from_rgb(0x3e, 0x2a, 0x1a);
pub const FRETBOARD_WOOD_DARK: egui::Color32 = egui::Color32::from_rgb(0x2a, 0x1c, 0x10);
pub const FRET_WIRE: egui::Color32 = egui::Color32::from_rgb(0xc0, 0xc0, 0xc0);
pub const STRING_COLOR: egui::Color32 = egui::Color32::from_rgb(0xd0, 0xd0, 0xd0);
pub const NUT_COLOR: egui::Color32 = egui::Color32::from_rgb(0xf0, 0xf0, 0xe0);
pub const FRET_MARKER: egui::Color32 = egui::Color32::from_rgb(0xe8, 0xe0, 0xd0);

// ── Accent colors ──
pub const ROOT_COLOR: egui::Color32 = egui::Color32::from_rgb(0xf0, 0xa5, 0x00);
pub const ROOT_GLOW: egui::Color32 = egui::Color32::from_rgba_premultiplied(0xf0, 0xa5, 0x00, 0x40);
pub const SCALE_TONE: egui::Color32 = egui::Color32::from_rgb(0x58, 0xa6, 0xff);
pub const SCALE_TONE_GLOW: egui::Color32 = egui::Color32::from_rgba_premultiplied(0x58, 0xa6, 0xff, 0x30);
pub const ACTIVE_NOTE: egui::Color32 = egui::Color32::from_rgb(0x3f, 0xb9, 0x50);
pub const ACTIVE_NOTE_GLOW: egui::Color32 = egui::Color32::from_rgba_premultiplied(0x3f, 0xb9, 0x50, 0x60);
pub const OUT_OF_KEY: egui::Color32 = egui::Color32::from_rgb(0xff, 0x55, 0x55);
pub const IN_KEY: egui::Color32 = egui::Color32::from_rgb(0x3f, 0xb9, 0x50);
pub const DIM_NOTE: egui::Color32 = egui::Color32::from_rgb(0x48, 0x4f, 0x58);

// ── Text colors ──
pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0xe6, 0xed, 0xf3);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(0x8b, 0x94, 0x9e);
pub const TEXT_ON_ACCENT: egui::Color32 = egui::Color32::from_rgb(0x0d, 0x11, 0x17);

// ── UI element colors ──
pub const BUTTON_BG: egui::Color32 = egui::Color32::from_rgb(0x30, 0x36, 0x3d);
pub const BUTTON_HOVER: egui::Color32 = egui::Color32::from_rgb(0x48, 0x4f, 0x58);
pub const BUTTON_ACTIVE: egui::Color32 = egui::Color32::from_rgb(0x58, 0xa6, 0xff);
pub const BORDER_COLOR: egui::Color32 = egui::Color32::from_rgb(0x30, 0x36, 0x3d);
pub const CONFIDENCE_BAR: egui::Color32 = egui::Color32::from_rgb(0x58, 0xa6, 0xff);
pub const LOCK_ACTIVE: egui::Color32 = egui::Color32::from_rgb(0xf0, 0xa5, 0x00);

// ── Dimensions ──
pub const FRETBOARD_HEIGHT: f32 = 240.0;
pub const NOTE_CIRCLE_RADIUS: f32 = 13.0;
pub const GLOW_RADIUS: f32 = 21.0;
pub const PANEL_PADDING: f32 = 14.0;
pub const PANEL_ROUNDING: u8 = 8;

/// Apply the Fretscope dark theme to an egui context
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let visuals = &mut style.visuals;

    visuals.dark_mode = true;
    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.panel_fill = BG_PANEL;
    visuals.window_fill = BG_DARK;
    visuals.extreme_bg_color = BG_DARK;
    visuals.faint_bg_color = BG_SURFACE;

    visuals.widgets.noninteractive.bg_fill = BG_SURFACE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(PANEL_ROUNDING);

    visuals.widgets.inactive.bg_fill = BUTTON_BG;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);

    visuals.widgets.hovered.bg_fill = BUTTON_HOVER;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);

    visuals.widgets.active.bg_fill = BUTTON_ACTIVE;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT_ON_ACCENT);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);

    visuals.selection.bg_fill = BUTTON_ACTIVE;
    visuals.selection.stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);

    visuals.window_corner_radius = egui::CornerRadius::same(PANEL_ROUNDING);
    visuals.window_stroke = egui::Stroke::new(1.0, BORDER_COLOR);

    ctx.set_style(style);
}
