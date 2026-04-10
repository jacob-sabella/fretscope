use nih_plug_egui::egui;
use std::sync::Arc;

use crate::key_detect::DetectedKey;
use crate::SharedState;

use super::theme;

pub struct NoteLogWidget;

impl NoteLogWidget {
    pub fn draw(
        ui: &mut egui::Ui,
        shared: &Arc<SharedState>,
        active_key: Option<&DetectedKey>,
    ) {
        egui::Frame::new()
            .fill(theme::BG_PANEL)
            .corner_radius(theme::PANEL_ROUNDING)
            .inner_margin(theme::PANEL_PADDING)
            .stroke(egui::Stroke::new(1.0, theme::BORDER_COLOR))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Header
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("NOTE LOG")
                                .color(theme::TEXT_SECONDARY)
                                .size(11.0),
                        );
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("Clear")
                                                .color(theme::TEXT_SECONDARY)
                                                .size(10.0),
                                        )
                                        .fill(theme::BUTTON_BG),
                                    )
                                    .clicked()
                                {
                                    shared.note_log.lock().unwrap().clear();
                                }
                            },
                        );
                    });

                    ui.add_space(4.0);

                    // Scrollable note list
                    let log = shared.note_log.lock().unwrap();
                    let entries = log.entries();

                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            if entries.is_empty() {
                                ui.label(
                                    egui::RichText::new("Play something...")
                                        .color(theme::TEXT_SECONDARY)
                                        .italics()
                                        .size(12.0),
                                );
                            }

                            for entry in entries.iter() {
                                // Phrase separator
                                if entry.is_phrase_start && !log.is_empty() {
                                    ui.add_space(2.0);
                                    ui.separator();
                                    ui.add_space(2.0);
                                }

                                let note_str = format!(
                                    "{}{}",
                                    entry.event.note_name, entry.event.octave
                                );

                                // Color based on whether note is in detected key
                                let color = if let Some(key) = active_key {
                                    let pc = entry.event.pitch_class();
                                    let interval = (pc + 12 - key.root_idx as u8) % 12;
                                    // Check if this pitch class is in the major/minor scale
                                    let scale_pcs: &[u8] = if key.is_major {
                                        &[0, 2, 4, 5, 7, 9, 11]
                                    } else {
                                        &[0, 2, 3, 5, 7, 8, 10]
                                    };
                                    if scale_pcs.contains(&interval) {
                                        theme::IN_KEY
                                    } else {
                                        theme::OUT_OF_KEY
                                    }
                                } else {
                                    theme::TEXT_PRIMARY
                                };

                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(&note_str)
                                            .color(color)
                                            .monospace()
                                            .size(13.0),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{:.0} Hz",
                                            entry.event.frequency
                                        ))
                                        .color(theme::TEXT_SECONDARY)
                                        .size(10.0),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{:.0}%",
                                            entry.event.confidence * 100.0
                                        ))
                                        .color(theme::TEXT_SECONDARY)
                                        .size(10.0),
                                    );
                                });
                            }
                        });
                });
            });
    }
}
