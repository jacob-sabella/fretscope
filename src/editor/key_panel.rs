use nih_plug_egui::egui;
use std::sync::Arc;

use crate::key_detect::DetectedKey;
use crate::theory::{self, ChordType, ScaleType};
use crate::{FretParams, SharedState};

use super::theme;

pub struct KeyPanel;

impl KeyPanel {
    pub fn draw(
        ui: &mut egui::Ui,
        shared: &Arc<SharedState>,
        _params: &Arc<FretParams>,
    ) {
        let detected_keys = {
            let kd = shared.key_detector.lock().unwrap();
            kd.detect()
        };

        let is_locked = shared
            .key_locked
            .load(std::sync::atomic::Ordering::Relaxed);
        let is_listening = shared
            .listening
            .load(std::sync::atomic::Ordering::Relaxed);
        let manual_root = shared
            .manual_root
            .load(std::sync::atomic::Ordering::Relaxed);

        let display_key: Option<DetectedKey> = if manual_root < 12 {
            // Manual root selected — build a synthetic key
            Some(DetectedKey {
                root: theory::all_note_names()[manual_root as usize].to_string(),
                root_idx: manual_root as usize,
                is_major: true,
                confidence: 1.0,
            })
        } else if is_locked {
            shared.locked_key.lock().unwrap().clone()
        } else {
            detected_keys.first().cloned()
        };

        egui::Frame::new()
            .fill(theme::BG_PANEL)
            .corner_radius(theme::PANEL_ROUNDING)
            .inner_margin(theme::PANEL_PADDING)
            .stroke(egui::Stroke::new(1.0, theme::BORDER_COLOR))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // ── Row 1: Listen toggle + Tuner ──
                    ui.horizontal(|ui| {
                        // Listen toggle
                        let listen_text = if is_listening { "● Listening" } else { "○ Paused" };
                        let listen_color = if is_listening {
                            theme::ACTIVE_NOTE
                        } else {
                            theme::TEXT_SECONDARY
                        };
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(listen_text)
                                        .color(listen_color)
                                        .size(12.0),
                                )
                                .fill(theme::BUTTON_BG),
                            )
                            .clicked()
                        {
                            shared
                                .listening
                                .store(!is_listening, std::sync::atomic::Ordering::Relaxed);
                        }

                        ui.add_space(12.0);

                        // Tuner display
                        let cents = shared
                            .cents_offset
                            .load(std::sync::atomic::Ordering::Relaxed);
                        let current_note = shared.current_note.lock().unwrap().clone();

                        ui.label(
                            egui::RichText::new("TUNER")
                                .color(theme::TEXT_SECONDARY)
                                .size(10.0),
                        );

                        if let Some(ref note) = current_note {
                            // Note name
                            ui.label(
                                egui::RichText::new(format!("{}{}", note.note_name, note.octave))
                                    .color(theme::ROOT_COLOR)
                                    .size(16.0)
                                    .strong()
                                    .monospace(),
                            );

                            // Cents indicator
                            let cents_color = if cents.abs() < 5.0 {
                                theme::IN_KEY // green — in tune
                            } else if cents.abs() < 15.0 {
                                theme::ROOT_COLOR // amber — close
                            } else {
                                theme::OUT_OF_KEY // red — out of tune
                            };

                            let cents_text = if cents > 0.0 {
                                format!("+{:.0}c", cents)
                            } else {
                                format!("{:.0}c", cents)
                            };

                            ui.label(
                                egui::RichText::new(cents_text)
                                    .color(cents_color)
                                    .size(14.0)
                                    .monospace(),
                            );

                            // Visual tuner bar
                            let bar_width = 80.0;
                            let bar_height = 8.0;
                            let bar_rect_pos = ui.available_rect_before_wrap();
                            let bar_rect = egui::Rect::from_min_size(
                                egui::Pos2::new(
                                    bar_rect_pos.left(),
                                    bar_rect_pos.center().y - bar_height / 2.0,
                                ),
                                egui::Vec2::new(bar_width, bar_height),
                            );
                            let painter = ui.painter();
                            painter.rect_filled(bar_rect, 3, theme::BG_ELEVATED);

                            // Center line
                            let center_x = bar_rect.center().x;
                            painter.line_segment(
                                [
                                    egui::Pos2::new(center_x, bar_rect.top()),
                                    egui::Pos2::new(center_x, bar_rect.bottom()),
                                ],
                                egui::Stroke::new(1.0, theme::TEXT_SECONDARY),
                            );

                            // Needle position (-50 to +50 cents mapped to bar width)
                            let needle_x =
                                center_x + (cents.clamp(-50.0, 50.0) / 50.0) * (bar_width / 2.0);
                            painter.circle_filled(
                                egui::Pos2::new(needle_x, bar_rect.center().y),
                                4.0,
                                cents_color,
                            );

                            ui.allocate_space(egui::Vec2::new(bar_width + 4.0, bar_height));
                        }
                    });

                    ui.add_space(6.0);

                    // ── Row 2: Key display ──
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("KEY")
                                .color(theme::TEXT_SECONDARY)
                                .size(10.0),
                        );

                        if let Some(ref key) = display_key {
                            if manual_root == 255 {
                                // Auto-detected key
                                ui.label(
                                    egui::RichText::new(key.display_name())
                                        .color(theme::ROOT_COLOR)
                                        .size(18.0)
                                        .strong(),
                                );

                                if key.confidence < 1.0 {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{:.0}%",
                                            key.confidence * 100.0
                                        ))
                                        .color(theme::TEXT_SECONDARY)
                                        .size(10.0),
                                    );
                                }
                            } else {
                                ui.label(
                                    egui::RichText::new(&key.root)
                                        .color(theme::ROOT_COLOR)
                                        .size(18.0)
                                        .strong(),
                                );
                                ui.label(
                                    egui::RichText::new("(manual)")
                                        .color(theme::TEXT_SECONDARY)
                                        .size(10.0),
                                );
                            }
                        } else {
                            ui.label(
                                egui::RichText::new("---")
                                    .color(theme::TEXT_SECONDARY)
                                    .size(18.0),
                            );
                        }
                    });

                    ui.add_space(4.0);

                    // ── Row 3: Root note selector + Lock ──
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Root:")
                                .color(theme::TEXT_SECONDARY)
                                .size(11.0),
                        );

                        // Auto button
                        let auto_selected = manual_root == 255;
                        let auto_color = if auto_selected {
                            theme::BUTTON_ACTIVE
                        } else {
                            theme::BUTTON_BG
                        };
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Auto")
                                        .color(if auto_selected {
                                            theme::TEXT_ON_ACCENT
                                        } else {
                                            theme::TEXT_PRIMARY
                                        })
                                        .size(11.0),
                                )
                                .fill(auto_color),
                            )
                            .clicked()
                        {
                            shared
                                .manual_root
                                .store(255, std::sync::atomic::Ordering::Relaxed);
                        }

                        // Note buttons
                        let note_names = theory::all_note_names();
                        for (i, name) in note_names.iter().enumerate() {
                            let selected = manual_root == i as u8;
                            let btn_color = if selected {
                                theme::ROOT_COLOR
                            } else {
                                theme::BUTTON_BG
                            };
                            let text_color = if selected {
                                theme::TEXT_ON_ACCENT
                            } else {
                                theme::TEXT_PRIMARY
                            };
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(*name)
                                            .color(text_color)
                                            .size(11.0),
                                    )
                                    .fill(btn_color)
                                    .min_size(egui::Vec2::new(24.0, 0.0)),
                                )
                                .clicked()
                            {
                                shared
                                    .manual_root
                                    .store(i as u8, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    });

                    ui.add_space(4.0);

                    // ── Row 4: Lock + Scale + Chord selectors ──
                    ui.horizontal(|ui| {
                        // Lock button (only relevant in auto mode)
                        if manual_root == 255 {
                            let lock_text = if is_locked { "🔒" } else { "🔓" };
                            let lock_color = if is_locked {
                                theme::LOCK_ACTIVE
                            } else {
                                theme::TEXT_SECONDARY
                            };
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(lock_text).color(lock_color),
                                    )
                                    .fill(theme::BUTTON_BG),
                                )
                                .clicked()
                            {
                                let new_locked = !is_locked;
                                shared
                                    .key_locked
                                    .store(new_locked, std::sync::atomic::Ordering::Relaxed);
                                if new_locked {
                                    *shared.locked_key.lock().unwrap() =
                                        detected_keys.first().cloned();
                                }
                            }

                            ui.add_space(8.0);
                        }

                        // Scale type dropdown
                        let mut scale_idx = shared
                            .selected_scale_idx
                            .load(std::sync::atomic::Ordering::Relaxed);

                        ui.label(
                            egui::RichText::new("Scale:")
                                .color(theme::TEXT_SECONDARY)
                                .size(11.0),
                        );

                        egui::ComboBox::from_id_salt("scale_select")
                            .selected_text(
                                egui::RichText::new(ScaleType::ALL[scale_idx].name())
                                    .color(theme::SCALE_TONE)
                                    .size(11.0),
                            )
                            .width(150.0)
                            .show_ui(ui, |ui| {
                                for (i, scale) in ScaleType::ALL.iter().enumerate() {
                                    if ui
                                        .selectable_value(&mut scale_idx, i, scale.name())
                                        .clicked()
                                    {
                                        shared.selected_scale_idx.store(
                                            i,
                                            std::sync::atomic::Ordering::Relaxed,
                                        );
                                    }
                                }
                            });

                        ui.add_space(12.0);

                        // Chord type dropdown
                        let mut voicing_idx = shared
                            .selected_voicing_idx
                            .load(std::sync::atomic::Ordering::Relaxed);

                        let chord_label = if let Some(ref key) = display_key {
                            format!("{}{}", key.root, ChordType::ALL[voicing_idx].suffix())
                        } else {
                            ChordType::ALL[voicing_idx].suffix().to_string()
                        };

                        ui.label(
                            egui::RichText::new("Chord:")
                                .color(theme::TEXT_SECONDARY)
                                .size(11.0),
                        );

                        egui::ComboBox::from_id_salt("chord_select")
                            .selected_text(
                                egui::RichText::new(&chord_label)
                                    .color(theme::SCALE_TONE)
                                    .size(11.0),
                            )
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                for (i, chord) in ChordType::ALL.iter().enumerate() {
                                    let label = if let Some(ref key) = display_key {
                                        format!("{}{}", key.root, chord.suffix())
                                    } else {
                                        chord.suffix().to_string()
                                    };
                                    if ui
                                        .selectable_value(&mut voicing_idx, i, &label)
                                        .clicked()
                                    {
                                        shared.selected_voicing_idx.store(
                                            i,
                                            std::sync::atomic::Ordering::Relaxed,
                                        );
                                    }
                                }
                            });
                    });

                    // ── Alternative keys (auto mode only) — clickable ──
                    if manual_root == 255 && detected_keys.len() > 1 {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Also:")
                                    .color(theme::TEXT_SECONDARY)
                                    .size(10.0),
                            );
                            for key in detected_keys.iter().skip(1) {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(format!(
                                                "{} ({:.0}%)",
                                                key.display_name(),
                                                key.confidence * 100.0
                                            ))
                                            .color(theme::DIM_NOTE)
                                            .size(10.0),
                                        )
                                        .fill(theme::BUTTON_BG),
                                    )
                                    .clicked()
                                {
                                    // Select this key: set manual root and lock
                                    shared.manual_root.store(
                                        key.root_idx as u8,
                                        std::sync::atomic::Ordering::Relaxed,
                                    );
                                }
                            }
                        });
                    }
                });
            });
    }
}
