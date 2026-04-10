use crate::pitch::NoteEvent;

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Krumhansl-Kessler key profiles (empirically derived from listening experiments)
const MAJOR_PROFILE: [f32; 12] = [
    6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
];

const MINOR_PROFILE: [f32; 12] = [
    6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
];

#[derive(Debug, Clone)]
pub struct DetectedKey {
    pub root: String,
    pub root_idx: usize,
    pub is_major: bool,
    pub confidence: f32,
}

impl DetectedKey {
    pub fn display_name(&self) -> String {
        format!(
            "{} {}",
            self.root,
            if self.is_major { "Major" } else { "Minor" }
        )
    }
}

pub struct KeyDetector {
    /// Pitch class histogram (12 bins, C=0)
    histogram: [f32; 12],
    /// Decay factor for older notes (applied each detection cycle)
    decay: f32,
    /// Number of notes processed
    note_count: u32,
}

impl KeyDetector {
    pub fn new() -> Self {
        Self {
            histogram: [0.0; 12],
            decay: 0.97,
            note_count: 0,
        }
    }

    pub fn add_note(&mut self, event: &NoteEvent) {
        // Apply decay to existing histogram
        for bin in &mut self.histogram {
            *bin *= self.decay;
        }

        // Add new note weighted by confidence
        let pc = event.pitch_class() as usize;
        self.histogram[pc] += event.confidence;
        self.note_count += 1;
    }

    /// Get the top detected keys using Krumhansl-Schmuckler correlation
    pub fn detect(&self) -> Vec<DetectedKey> {
        if self.note_count < 3 {
            return Vec::new();
        }

        let mut candidates: Vec<DetectedKey> = Vec::with_capacity(24);

        for root in 0..12 {
            // Rotate histogram so that `root` is at index 0
            let rotated: Vec<f32> = (0..12).map(|i| self.histogram[(i + root) % 12]).collect();

            let major_corr = pearson_correlation(&rotated, &MAJOR_PROFILE);
            let minor_corr = pearson_correlation(&rotated, &MINOR_PROFILE);

            candidates.push(DetectedKey {
                root: NOTE_NAMES[root].to_string(),
                root_idx: root,
                is_major: true,
                confidence: major_corr,
            });
            candidates.push(DetectedKey {
                root: NOTE_NAMES[root].to_string(),
                root_idx: root,
                is_major: false,
                confidence: minor_corr,
            });
        }

        candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        candidates.truncate(3);
        candidates
    }

    pub fn reset(&mut self) {
        self.histogram = [0.0; 12];
        self.note_count = 0;
    }
}

fn pearson_correlation(x: &[f32], y: &[f32]) -> f32 {
    let n = x.len() as f32;
    let sum_x: f32 = x.iter().sum();
    let sum_y: f32 = y.iter().sum();
    let sum_xy: f32 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    let sum_x2: f32 = x.iter().map(|a| a * a).sum();
    let sum_y2: f32 = y.iter().map(|a| a * a).sum();

    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();

    if denominator.abs() < 1e-10 {
        0.0
    } else {
        numerator / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_major_detection() {
        let mut kd = KeyDetector::new();
        // Feed C major scale notes: C D E F G A B
        let c_major_pcs: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];
        for &pc in c_major_pcs.iter().cycle().take(21) {
            let event = NoteEvent {
                note_name: NOTE_NAMES[pc as usize].to_string(),
                octave: 4,
                midi_note: 60 + pc,
                frequency: 440.0,
                confidence: 0.95,
                timestamp: std::time::Instant::now(),
            };
            kd.add_note(&event);
        }

        let keys = kd.detect();
        assert!(!keys.is_empty());
        assert_eq!(keys[0].root, "C");
        assert!(keys[0].is_major);
    }
}
