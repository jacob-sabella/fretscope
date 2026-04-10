use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, resizable_window::ResizableWindow, EguiState};
use std::sync::Arc;

use crate::key_detect::DetectedKey;
use crate::theory::{self, ScaleType};
use crate::{FretParams, SharedState};

mod fretboard;
mod key_panel;
mod note_log_widget;
pub mod theme;

const WINDOW_WIDTH: u32 = 1050;
const WINDOW_HEIGHT: u32 = 720;
const MIN_WIDTH: f32 = 800.0;
const MIN_HEIGHT: f32 = 550.0;

struct EditorState {
    fretboard_widget: fretboard::FretboardWidget,
}

pub fn create(
    shared: Arc<SharedState>,
    params: Arc<FretParams>,
) -> Option<Box<dyn Editor>> {
    let egui_state = EguiState::from_size(WINDOW_WIDTH, WINDOW_HEIGHT);
    let egui_state_clone = egui_state.clone();

    let editor_state = Arc::new(std::sync::Mutex::new(EditorState {
        fretboard_widget: fretboard::FretboardWidget::new(),
    }));

    create_egui_editor(
        egui_state,
        (),
        |ctx, _state| {
            theme::apply_theme(ctx);
        },
        move |ctx, _setter, _state| {
            let current_note = shared.current_note.lock().unwrap().clone();
            let is_locked = shared
                .key_locked
                .load(std::sync::atomic::Ordering::Relaxed);
            let scale_idx = shared
                .selected_scale_idx
                .load(std::sync::atomic::Ordering::Relaxed);
            let current_scale = ScaleType::ALL[scale_idx];
            let manual_root = shared
                .manual_root
                .load(std::sync::atomic::Ordering::Relaxed);
            let tuning_idx = shared
                .tuning_idx
                .load(std::sync::atomic::Ordering::Relaxed);
            let tunings = theory::preset_tunings();
            let tuning = if tuning_idx == usize::MAX {
                // Custom tuning
                shared
                    .custom_tuning
                    .lock()
                    .unwrap()
                    .clone()
                    .unwrap_or_else(|| tunings[0].clone())
            } else {
                tunings[tuning_idx.min(tunings.len() - 1)].clone()
            };
            let display_frets = shared
                .display_frets
                .load(std::sync::atomic::Ordering::Relaxed);

            let display_key: Option<DetectedKey> = if manual_root < 12 {
                Some(DetectedKey {
                    root: theory::all_note_names()[manual_root as usize].to_string(),
                    root_idx: manual_root as usize,
                    is_major: true,
                    confidence: 1.0,
                })
            } else if is_locked {
                shared.locked_key.lock().unwrap().clone()
            } else {
                let kd = shared.key_detector.lock().unwrap();
                kd.detect().into_iter().next()
            };

            let scale_positions = if let Some(ref key) = display_key {
                theory::scale_positions(key.root_idx as u8, current_scale, &tuning, display_frets)
            } else {
                Vec::new()
            };

            {
                let mut es = editor_state.lock().unwrap();
                es.fretboard_widget
                    .set_active_note(current_note.as_ref().map(|n| n.midi_note));
            }

            let display_opts = fretboard::DisplayOptions {
                flipped: shared
                    .fretboard_flipped
                    .load(std::sync::atomic::Ordering::Relaxed),
                show_note_names: shared
                    .show_note_names
                    .load(std::sync::atomic::Ordering::Relaxed),
                show_glow: shared
                    .show_glow
                    .load(std::sync::atomic::Ordering::Relaxed),
                show_fret_numbers: shared
                    .show_fret_numbers
                    .load(std::sync::atomic::Ordering::Relaxed),
                degree_mask: shared
                    .degree_mask
                    .load(std::sync::atomic::Ordering::Relaxed),
                note_label_mode: shared
                    .note_label_mode
                    .load(std::sync::atomic::Ordering::Relaxed),
                show_open_fret: shared
                    .show_open_fret
                    .load(std::sync::atomic::Ordering::Relaxed),
                display_frets,
                num_strings: tuning.string_count() as u8,
            };

            // Spacebar toggles listen/pause
            if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                let was_listening = shared
                    .listening
                    .load(std::sync::atomic::Ordering::Relaxed);
                shared
                    .listening
                    .store(!was_listening, std::sync::atomic::Ordering::Relaxed);
            }

            // Resizable window with drag corner
            ResizableWindow::new("fretscope_main")
                .min_size(egui::Vec2::new(MIN_WIDTH, MIN_HEIGHT))
                .show(ctx, &egui_state_clone, |ui| {
                    let frame = egui::Frame::new()
                        .fill(theme::BG_DARK)
                        .inner_margin(16.0);

                    frame.show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 6.0);

                        // ── Title bar ──
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("FRETSCOPE")
                                    .color(theme::ROOT_COLOR)
                                    .size(20.0)
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new("v0.1.0")
                                    .color(theme::TEXT_SECONDARY)
                                    .size(11.0),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if let Some(ref note) = current_note {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{}{}  {:.0} Hz",
                                                note.note_name, note.octave, note.frequency
                                            ))
                                            .color(theme::ACTIVE_NOTE)
                                            .size(16.0)
                                            .strong(),
                                        );
                                    }
                                },
                            );
                        });

                        ui.add_space(6.0);

                        // ── View options row 1 ──
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;

                            ui.label(
                                egui::RichText::new("View:")
                                    .color(theme::TEXT_SECONDARY)
                                    .size(11.0),
                            );

                            let toggle_btn =
                                |ui: &mut egui::Ui,
                                 label: &str,
                                 active: bool,
                                 shared_flag: &std::sync::atomic::AtomicBool| {
                                    let color = if active {
                                        theme::BUTTON_ACTIVE
                                    } else {
                                        theme::BUTTON_BG
                                    };
                                    let text_color = if active {
                                        theme::TEXT_ON_ACCENT
                                    } else {
                                        theme::TEXT_SECONDARY
                                    };
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(label)
                                                    .color(text_color)
                                                    .size(11.0),
                                            )
                                            .fill(color),
                                        )
                                        .clicked()
                                    {
                                        shared_flag.store(
                                            !active,
                                            std::sync::atomic::Ordering::Relaxed,
                                        );
                                    }
                                };

                            toggle_btn(ui, "Flip", display_opts.flipped, &shared.fretboard_flipped);
                            toggle_btn(
                                ui,
                                "Notes",
                                display_opts.show_note_names,
                                &shared.show_note_names,
                            );
                            toggle_btn(ui, "Glow", display_opts.show_glow, &shared.show_glow);
                            toggle_btn(
                                ui,
                                "Fret #",
                                display_opts.show_fret_numbers,
                                &shared.show_fret_numbers,
                            );

                            // Open fret toggle
                            let open_active = display_opts.show_open_fret;
                            let open_color = if open_active {
                                theme::BUTTON_ACTIVE
                            } else {
                                theme::BUTTON_BG
                            };
                            let open_text_color = if open_active {
                                theme::TEXT_ON_ACCENT
                            } else {
                                theme::TEXT_SECONDARY
                            };
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("Open Fret")
                                            .color(open_text_color)
                                            .size(11.0),
                                    )
                                    .fill(open_color),
                                )
                                .clicked()
                            {
                                shared.show_open_fret.store(
                                    !open_active,
                                    std::sync::atomic::Ordering::Relaxed,
                                );
                            }

                            // Label mode
                            let label_mode = display_opts.note_label_mode;
                            let mode_text = match label_mode {
                                0 => "Label: ABC",
                                1 => "Label: 123",
                                _ => "Label: A/1",
                            };
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(mode_text)
                                            .color(theme::SCALE_TONE)
                                            .size(11.0),
                                    )
                                    .fill(theme::BUTTON_BG),
                                )
                                .clicked()
                            {
                                let new_mode = (label_mode + 1) % 3;
                                shared
                                    .note_label_mode
                                    .store(new_mode, std::sync::atomic::Ordering::Relaxed);
                            }

                            ui.separator();

                            // Scale degree toggles
                            ui.label(
                                egui::RichText::new("Degrees:")
                                    .color(theme::TEXT_SECONDARY)
                                    .size(11.0),
                            );

                            let num_degrees = current_scale.intervals().len();
                            let degree_labels = ["1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th"];
                            let mask = display_opts.degree_mask;

                            for deg in 0..num_degrees.min(8) {
                                let bit = 1u8 << deg;
                                let active = mask & bit != 0;
                                let color = if active {
                                    if deg == 0 {
                                        theme::ROOT_COLOR
                                    } else {
                                        theme::SCALE_TONE
                                    }
                                } else {
                                    theme::BUTTON_BG
                                };
                                let text_color = if active {
                                    theme::TEXT_ON_ACCENT
                                } else {
                                    theme::TEXT_SECONDARY
                                };
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(degree_labels[deg])
                                                .color(text_color)
                                                .size(10.0),
                                        )
                                        .fill(color)
                                        .min_size(egui::Vec2::new(24.0, 0.0)),
                                    )
                                    .clicked()
                                {
                                    let new_mask = mask ^ bit;
                                    shared.degree_mask.store(
                                        new_mask,
                                        std::sync::atomic::Ordering::Relaxed,
                                    );
                                }
                            }
                        });

                        ui.add_space(2.0);

                        // ── View options row 2: tuning + frets ──
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Tuning:")
                                    .color(theme::TEXT_SECONDARY)
                                    .size(11.0),
                            );
                            let tuning_label = if tuning_idx == usize::MAX {
                                "Custom".to_string()
                            } else {
                                tuning.name.clone()
                            };
                            egui::ComboBox::from_id_salt("tuning_select")
                                .selected_text(
                                    egui::RichText::new(&tuning_label)
                                        .color(theme::SCALE_TONE)
                                        .size(11.0),
                                )
                                .width(200.0)
                                .show_ui(ui, |ui| {
                                    for (i, t) in tunings.iter().enumerate() {
                                        let mut sel = tuning_idx;
                                        if ui
                                            .selectable_value(&mut sel, i, t.name.as_str())
                                            .clicked()
                                        {
                                            shared.tuning_idx.store(
                                                i,
                                                std::sync::atomic::Ordering::Relaxed,
                                            );
                                        }
                                    }
                                    // Custom option
                                    let mut sel = tuning_idx;
                                    if ui
                                        .selectable_value(&mut sel, usize::MAX, "Custom...")
                                        .clicked()
                                    {
                                        // Initialize custom tuning from current tuning
                                        let mut ct = shared.custom_tuning.lock().unwrap();
                                        if ct.is_none() {
                                            *ct = Some(tuning.clone());
                                        }
                                        shared.tuning_idx.store(
                                            usize::MAX,
                                            std::sync::atomic::Ordering::Relaxed,
                                        );
                                    }
                                });

                            ui.add_space(12.0);

                            ui.label(
                                egui::RichText::new("Frets:")
                                    .color(theme::TEXT_SECONDARY)
                                    .size(11.0),
                            );
                            let mut frets_val = display_frets as f32;
                            let slider = egui::Slider::new(&mut frets_val, 8.0..=30.0)
                                .step_by(1.0)
                                .show_value(true);
                            if ui.add(slider).changed() {
                                shared.display_frets.store(
                                    frets_val as u8,
                                    std::sync::atomic::Ordering::Relaxed,
                                );
                            }
                        });

                        // ── Custom tuning editor (shown when Custom is selected) ──
                        if tuning_idx == usize::MAX {
                            ui.add_space(2.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                ui.label(
                                    egui::RichText::new("Custom:")
                                        .color(theme::TEXT_SECONDARY)
                                        .size(11.0),
                                );

                                // String count +/- buttons
                                let ct = shared.custom_tuning.lock().unwrap().clone();
                                let mut ct = ct.unwrap_or_else(|| tunings[0].clone());

                                ui.label(
                                    egui::RichText::new("Strings:")
                                        .color(theme::TEXT_SECONDARY)
                                        .size(10.0),
                                );
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("-").size(12.0),
                                        )
                                        .fill(theme::BUTTON_BG)
                                        .min_size(egui::Vec2::new(20.0, 0.0)),
                                    )
                                    .clicked()
                                    && ct.notes.len() > 1
                                {
                                    ct.notes.pop();
                                    ct.name = "Custom".into();
                                    *shared.custom_tuning.lock().unwrap() = Some(ct.clone());
                                }
                                ui.label(
                                    egui::RichText::new(format!("{}", ct.notes.len()))
                                        .color(theme::ROOT_COLOR)
                                        .size(12.0)
                                        .strong(),
                                );
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("+").size(12.0),
                                        )
                                        .fill(theme::BUTTON_BG)
                                        .min_size(egui::Vec2::new(20.0, 0.0)),
                                    )
                                    .clicked()
                                    && ct.notes.len() < 12
                                {
                                    // Add a string 5 semitones above the last
                                    let last = *ct.notes.last().unwrap_or(&64);
                                    ct.notes.push((last + 5).min(127));
                                    ct.name = "Custom".into();
                                    *shared.custom_tuning.lock().unwrap() = Some(ct.clone());
                                }

                                ui.separator();

                                // Per-string note selectors
                                for (i, midi) in ct.notes.clone().iter().enumerate() {
                                    let label = theory::midi_to_label(*midi);
                                    ui.label(
                                        egui::RichText::new(format!("S{}:", i + 1))
                                            .color(theme::TEXT_SECONDARY)
                                            .size(9.0),
                                    );

                                    // Down button
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("▼").size(9.0),
                                            )
                                            .fill(theme::BUTTON_BG),
                                        )
                                        .clicked()
                                        && *midi > 20
                                    {
                                        let mut new_ct = ct.clone();
                                        new_ct.notes[i] -= 1;
                                        new_ct.name = "Custom".into();
                                        *shared.custom_tuning.lock().unwrap() =
                                            Some(new_ct);
                                    }

                                    ui.label(
                                        egui::RichText::new(&label)
                                            .color(theme::SCALE_TONE)
                                            .size(10.0)
                                            .monospace(),
                                    );

                                    // Up button
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("▲").size(9.0),
                                            )
                                            .fill(theme::BUTTON_BG),
                                        )
                                        .clicked()
                                        && *midi < 127
                                    {
                                        let mut new_ct = ct.clone();
                                        new_ct.notes[i] += 1;
                                        new_ct.name = "Custom".into();
                                        *shared.custom_tuning.lock().unwrap() =
                                            Some(new_ct);
                                    }
                                }
                            });
                        }

                        ui.add_space(6.0);

                        // ── Fretboard (fills available width) ──
                        {
                            let mut es = editor_state.lock().unwrap();
                            es.fretboard_widget
                                .draw(ui, &scale_positions, 0, &display_opts);
                        }

                        ui.add_space(10.0);

                        // ── Bottom panels — use available remaining height ──
                        let remaining = ui.available_height() - 10.0;
                        let panel_height = remaining.max(150.0);

                        ui.horizontal(|ui| {
                            let half_width = (ui.available_width() - 12.0) / 2.0;

                            ui.vertical(|ui| {
                                ui.set_width(half_width);
                                ui.set_min_height(panel_height);
                                key_panel::KeyPanel::draw(ui, &shared, &params);
                            });

                            ui.add_space(12.0);

                            ui.vertical(|ui| {
                                ui.set_width(half_width);
                                ui.set_min_height(panel_height);
                                note_log_widget::NoteLogWidget::draw(
                                    ui,
                                    &shared,
                                    display_key.as_ref(),
                                );
                            });
                        });
                    });
                });
        },
    )
}
