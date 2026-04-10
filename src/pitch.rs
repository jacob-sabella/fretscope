use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector as PitchDetectorTrait;

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// How many samples to accumulate before running detection.
/// 2048 @ 44100 Hz ≈ 46ms — good balance of latency vs accuracy for guitar.
const DETECTION_WINDOW: usize = 2048;

/// How many new samples before we run detection again (overlap).
/// 1024 means 50% overlap — detection runs roughly every 23ms.
const HOP_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct NoteEvent {
    pub note_name: String,
    pub octave: i32,
    pub midi_note: u8,
    pub frequency: f32,
    pub confidence: f32,
    pub timestamp: std::time::Instant,
}

impl NoteEvent {
    /// Pitch class (0-11, C=0)
    pub fn pitch_class(&self) -> u8 {
        self.midi_note % 12
    }
}

pub struct PitchDetector {
    sample_rate: f32,
    /// Accumulation buffer for incoming samples
    buffer: Vec<f32>,
    /// How many new samples we've received since last detection
    samples_since_detect: usize,
}

impl PitchDetector {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            buffer: Vec::with_capacity(DETECTION_WINDOW * 2),
            samples_since_detect: 0,
        }
    }

    /// Feed samples from the audio callback. Returns a NoteEvent if pitch was detected.
    /// Call this every process() block — it accumulates internally and runs
    /// detection when enough samples have been gathered.
    pub fn feed(&mut self, samples: &[f32], confidence_threshold: f32) -> Option<NoteEvent> {
        // Append new samples
        self.buffer.extend_from_slice(samples);
        self.samples_since_detect += samples.len();

        // Not enough samples yet
        if self.buffer.len() < DETECTION_WINDOW || self.samples_since_detect < HOP_SIZE {
            return None;
        }

        // Take the last DETECTION_WINDOW samples for analysis
        let start = self.buffer.len() - DETECTION_WINDOW;
        let window = &self.buffer[start..];

        let result = self.run_detection(window, confidence_threshold);

        // Reset hop counter
        self.samples_since_detect = 0;

        // Keep buffer from growing forever — retain only what we need for overlap
        if self.buffer.len() > DETECTION_WINDOW * 2 {
            let drain_to = self.buffer.len() - DETECTION_WINDOW;
            self.buffer.drain(..drain_to);
        }

        result
    }

    fn run_detection(&self, samples: &[f32], confidence_threshold: f32) -> Option<NoteEvent> {
        let size = samples.len();
        let padding = size / 2;

        let mut detector = McLeodDetector::new(size, padding);
        let pitch = detector.get_pitch(samples, self.sample_rate as usize, 0.5, 0.3)?;

        let frequency = pitch.frequency as f32;
        let confidence = pitch.clarity as f32;

        if confidence < confidence_threshold || frequency < 50.0 || frequency > 2000.0 {
            return None;
        }

        // Convert frequency to MIDI note number
        let midi_float = 12.0 * (frequency / 440.0).log2() + 69.0;
        if midi_float < 0.0 || midi_float > 127.0 {
            return None;
        }
        let midi_note = midi_float.round() as u8;
        let note_idx = (midi_note % 12) as usize;
        let octave = (midi_note as i32 / 12) - 1;

        Some(NoteEvent {
            note_name: NOTE_NAMES[note_idx].to_string(),
            octave,
            midi_note,
            frequency,
            confidence,
            timestamp: std::time::Instant::now(),
        })
    }
}
