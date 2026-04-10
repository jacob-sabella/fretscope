use nih_plug_egui::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

use crate::theory::{self, FretPosition};

use super::theme;

const SINGLE_DOT_FRETS: [u8; 10] = [3, 5, 7, 9, 15, 17, 19, 21, 24, 27];
const DOUBLE_DOT_FRETS: [u8; 2] = [12, 24];

pub struct DisplayOptions {
    pub flipped: bool,
    pub show_note_names: bool,
    pub show_glow: bool,
    pub show_fret_numbers: bool,
    /// Bitmask of which scale degrees to show (bit 0 = 1st, bit 1 = 2nd, etc.)
    pub degree_mask: u8,
    /// 0 = note letter, 1 = scale degree, 2 = both
    pub note_label_mode: u8,
    /// Show fret 0 as a column on the board
    pub show_open_fret: bool,
    /// Number of frets to display
    pub display_frets: u8,
    /// Number of strings in current tuning
    pub num_strings: u8,
}

pub struct FretboardWidget {
    active_midi: Option<u8>,
    pulse_phase: f32,
}

impl FretboardWidget {
    pub fn new() -> Self {
        Self {
            active_midi: None,
            pulse_phase: 0.0,
        }
    }

    pub fn set_active_note(&mut self, midi: Option<u8>) {
        if self.active_midi != midi {
            self.active_midi = midi;
            self.pulse_phase = 0.0;
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        scale_positions: &[FretPosition],
        start_fret: u8,
        opts: &DisplayOptions,
    ) {
        let available = ui.available_size();
        let width = available.x;
        // Scale height with string count
        let base_height = theme::FRETBOARD_HEIGHT;
        let height = base_height * (opts.num_strings as f32 / 6.0).max(0.7);
        let (response, painter) =
            ui.allocate_painter(Vec2::new(width, height), egui::Sense::hover());
        let rect = response.rect;

        self.pulse_phase = (self.pulse_phase + 0.03) % 1.0;

        let display_frets = opts.display_frets as f32;
        let num_strings = opts.num_strings as f32;
        let left_margin = 40.0;
        let right_margin = 20.0;
        let top_margin = 20.0;
        let bottom_margin = 20.0;

        // If show_open_fret, fret 0 takes a column
        let total_columns = if opts.show_open_fret {
            display_frets + 1.0
        } else {
            display_frets
        };

        let fret_area_width = width - left_margin - right_margin;
        let fret_area_height = height - top_margin - bottom_margin;
        let fret_spacing = fret_area_width / total_columns;
        let string_spacing = if num_strings > 1.0 {
            fret_area_height / (num_strings - 1.0)
        } else {
            fret_area_height
        };

        // String Y position
        let string_y = |string_idx: u8| -> f32 {
            let s = if opts.flipped {
                (opts.num_strings - 1) - string_idx
            } else {
                string_idx
            };
            rect.top() + top_margin + s as f32 * string_spacing
        };

        // Fret X position (fret_num is 0-based display column)
        let fret_x = |display_fret: u8| -> f32 {
            let col = if opts.show_open_fret {
                display_fret as f32 + 0.5
            } else if display_fret == 0 {
                // Open string drawn left of nut
                return rect.left() + left_margin - 18.0;
            } else {
                display_fret as f32 - 0.5
            };
            rect.left() + left_margin + col * fret_spacing
        };

        // ── Fretboard background ──
        let fb_rect = Rect::from_min_size(
            Pos2::new(rect.left() + left_margin, rect.top() + top_margin),
            Vec2::new(fret_area_width, fret_area_height),
        );
        painter.rect_filled(fb_rect.expand(2.0), 4, theme::FRETBOARD_WOOD_DARK);
        painter.rect_filled(fb_rect, 4, theme::FRETBOARD_WOOD);

        // ── Nut ──
        let nut_col = if opts.show_open_fret { 1.0 } else { 0.0 };
        if start_fret == 0 {
            let nut_x = rect.left() + left_margin + nut_col * fret_spacing;
            painter.line_segment(
                [
                    Pos2::new(nut_x, rect.top() + top_margin - 2.0),
                    Pos2::new(nut_x, rect.top() + top_margin + fret_area_height + 2.0),
                ],
                Stroke::new(5.0, theme::NUT_COLOR),
            );
        }

        // ── Fret wires ──
        let fret_wire_start = if opts.show_open_fret { 2 } else { 1 };
        for fret in fret_wire_start..=opts.display_frets {
            let col = if opts.show_open_fret {
                fret as f32
            } else {
                fret as f32
            };
            let x = rect.left() + left_margin + col * fret_spacing;
            painter.line_segment(
                [
                    Pos2::new(x, rect.top() + top_margin),
                    Pos2::new(x, rect.top() + top_margin + fret_area_height),
                ],
                Stroke::new(2.0, theme::FRET_WIRE),
            );
        }

        // ── Strings ──
        for s in 0..opts.num_strings {
            let y = string_y(s);
            let thickness = 1.0 + ((opts.num_strings - 1) - s) as f32 * 0.4;
            painter.line_segment(
                [
                    Pos2::new(rect.left() + left_margin, y),
                    Pos2::new(rect.left() + left_margin + fret_area_width, y),
                ],
                Stroke::new(thickness, theme::STRING_COLOR),
            );
        }

        // ── Fret markers ──
        let marker_y_center = rect.top() + top_margin + fret_area_height / 2.0;
        for &fret in &SINGLE_DOT_FRETS {
            if fret > start_fret && fret <= start_fret + opts.display_frets {
                let df = fret - start_fret;
                let col = if opts.show_open_fret {
                    df as f32 + 0.5
                } else {
                    df as f32 - 0.5
                };
                let x = rect.left() + left_margin + col * fret_spacing;
                painter.circle_filled(Pos2::new(x, marker_y_center), 5.0, theme::FRET_MARKER);
            }
        }
        for &fret in &DOUBLE_DOT_FRETS {
            if fret > start_fret && fret <= start_fret + opts.display_frets {
                let df = fret - start_fret;
                let col = if opts.show_open_fret {
                    df as f32 + 0.5
                } else {
                    df as f32 - 0.5
                };
                let x = rect.left() + left_margin + col * fret_spacing;
                let offset = string_spacing * 1.2;
                painter.circle_filled(
                    Pos2::new(x, marker_y_center - offset),
                    5.0,
                    theme::FRET_MARKER,
                );
                painter.circle_filled(
                    Pos2::new(x, marker_y_center + offset),
                    5.0,
                    theme::FRET_MARKER,
                );
            }
        }

        // ── Fret numbers ──
        if opts.show_fret_numbers {
            let start = if opts.show_open_fret { 0 } else { 1 };
            for fret in start..=opts.display_frets {
                let actual_fret = start_fret + fret;
                let col = if opts.show_open_fret {
                    fret as f32 + 0.5
                } else {
                    if fret == 0 { continue; }
                    fret as f32 - 0.5
                };
                let x = rect.left() + left_margin + col * fret_spacing;
                let y = rect.top() + top_margin + fret_area_height + 14.0;
                painter.text(
                    Pos2::new(x, y),
                    egui::Align2::CENTER_CENTER,
                    actual_fret.to_string(),
                    egui::FontId::proportional(10.0),
                    theme::TEXT_SECONDARY,
                );
            }
        }

        // ── Scale/chord note positions ──
        for pos in scale_positions {
            if pos.fret < start_fret || pos.fret > start_fret + opts.display_frets {
                continue;
            }
            if pos.string >= opts.num_strings {
                continue;
            }

            // Filter by scale degree mask
            if opts.degree_mask != 0xFF && pos.scale_degree > 0 {
                let bit = 1u8 << (pos.scale_degree - 1);
                if opts.degree_mask & bit == 0 {
                    continue;
                }
            }

            let display_fret = pos.fret - start_fret;
            let x = fret_x(display_fret);
            let y = string_y(pos.string);

            let (fill_color, glow_color) = if pos.is_root {
                (theme::ROOT_COLOR, theme::ROOT_GLOW)
            } else {
                (theme::SCALE_TONE, theme::SCALE_TONE_GLOW)
            };

            let is_active = self.active_midi == Some(pos.midi_note);
            let (fill, glow) = if is_active {
                (theme::ACTIVE_NOTE, theme::ACTIVE_NOTE_GLOW)
            } else {
                (fill_color, glow_color)
            };

            if opts.show_glow {
                let glow_radius = if is_active {
                    theme::GLOW_RADIUS + (self.pulse_phase * std::f32::consts::TAU).sin() * 4.0
                } else {
                    theme::GLOW_RADIUS
                };
                painter.circle_filled(Pos2::new(x, y), glow_radius, glow);
            }

            let radius = if is_active {
                theme::NOTE_CIRCLE_RADIUS + 2.0
            } else {
                theme::NOTE_CIRCLE_RADIUS
            };
            painter.circle_filled(Pos2::new(x, y), radius, fill);

            // Note label
            if opts.show_note_names {
                let label = match opts.note_label_mode {
                    0 => pos.note_name.clone(),
                    1 => theory::degree_label(pos.scale_degree).to_string(),
                    2 => format!("{}\n{}", pos.note_name, theory::degree_label(pos.scale_degree)),
                    _ => pos.note_name.clone(),
                };

                let text_color = if pos.is_root || is_active {
                    theme::TEXT_ON_ACCENT
                } else {
                    Color32::WHITE
                };

                let font_size = if opts.note_label_mode == 2 { 8.0 } else { 10.0 };
                painter.text(
                    Pos2::new(x, y),
                    egui::Align2::CENTER_CENTER,
                    &label,
                    egui::FontId::monospace(font_size),
                    text_color,
                );
            }
        }

        if self.active_midi.is_some() {
            ui.ctx().request_repaint();
        }
    }
}
