use nih_plug::prelude::*;
use std::sync::{Arc, Mutex};

mod editor;
mod key_detect;
mod note_log;
mod pitch;
mod theory;

pub struct Fretscope {
    params: Arc<FretParams>,
    pitch_detector: pitch::PitchDetector,
    /// Shared state between audio thread and GUI
    shared_state: Arc<SharedState>,
}

/// Data shared between the audio processing thread and the GUI.
/// The audio thread writes, the GUI thread reads.
pub struct SharedState {
    /// Latest detected note
    pub current_note: Mutex<Option<pitch::NoteEvent>>,
    /// Key detector state
    pub key_detector: Mutex<key_detect::KeyDetector>,
    /// Note log history
    pub note_log: Mutex<note_log::NoteLog>,
    /// Whether key is locked by user
    pub key_locked: std::sync::atomic::AtomicBool,
    /// The locked key (if any)
    pub locked_key: Mutex<Option<key_detect::DetectedKey>>,
    /// Currently selected scale type index
    pub selected_scale_idx: std::sync::atomic::AtomicUsize,
    /// Currently selected mode index
    pub selected_mode_idx: std::sync::atomic::AtomicUsize,
    /// Currently selected chord voicing index
    pub selected_voicing_idx: std::sync::atomic::AtomicUsize,
    /// Whether pitch detection is active
    pub listening: std::sync::atomic::AtomicBool,
    /// Manually selected root note (0-11, 255 = auto/detect)
    pub manual_root: std::sync::atomic::AtomicU8,
    /// Cents offset from nearest note (for tuner display)
    pub cents_offset: atomic_float::AtomicF32,
    /// Flip fretboard (low E on top vs bottom)
    pub fretboard_flipped: std::sync::atomic::AtomicBool,
    /// Show note names on fretboard dots
    pub show_note_names: std::sync::atomic::AtomicBool,
    /// Show glow effects on fretboard
    pub show_glow: std::sync::atomic::AtomicBool,
    /// Show fret numbers below fretboard
    pub show_fret_numbers: std::sync::atomic::AtomicBool,
    /// Which scale degrees to display (bitmask: bit 0 = 1st, bit 1 = 2nd, ..., bit 6 = 7th)
    /// Default: all on (0xFF)
    pub degree_mask: std::sync::atomic::AtomicU8,
    /// Note label mode: 0 = note letter, 1 = scale degree, 2 = both
    pub note_label_mode: std::sync::atomic::AtomicU8,
    /// Show fret 0 (open) as its own column on the fretboard
    pub show_open_fret: std::sync::atomic::AtomicBool,
    /// Max frets to display
    pub display_frets: std::sync::atomic::AtomicU8,
    /// Currently selected tuning preset index (usize::MAX = custom)
    pub tuning_idx: std::sync::atomic::AtomicUsize,
    /// Custom tuning (user-defined string count + notes)
    pub custom_tuning: Mutex<Option<theory::Tuning>>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            current_note: Mutex::new(None),
            key_detector: Mutex::new(key_detect::KeyDetector::new()),
            note_log: Mutex::new(note_log::NoteLog::new()),
            key_locked: std::sync::atomic::AtomicBool::new(false),
            locked_key: Mutex::new(None),
            selected_scale_idx: std::sync::atomic::AtomicUsize::new(0),
            selected_mode_idx: std::sync::atomic::AtomicUsize::new(0),
            selected_voicing_idx: std::sync::atomic::AtomicUsize::new(0),
            listening: std::sync::atomic::AtomicBool::new(true),
            manual_root: std::sync::atomic::AtomicU8::new(255), // 255 = auto
            cents_offset: atomic_float::AtomicF32::new(0.0),
            fretboard_flipped: std::sync::atomic::AtomicBool::new(false),
            show_note_names: std::sync::atomic::AtomicBool::new(true),
            show_glow: std::sync::atomic::AtomicBool::new(true),
            show_fret_numbers: std::sync::atomic::AtomicBool::new(true),
            degree_mask: std::sync::atomic::AtomicU8::new(0xFF), // all degrees on
            note_label_mode: std::sync::atomic::AtomicU8::new(0), // 0=letter
            show_open_fret: std::sync::atomic::AtomicBool::new(false),
            display_frets: std::sync::atomic::AtomicU8::new(16),
            tuning_idx: std::sync::atomic::AtomicUsize::new(0),
            custom_tuning: Mutex::new(None),
        }
    }
}

#[derive(Params)]
pub struct FretParams {
    #[id = "confidence"]
    pub confidence_threshold: FloatParam,
}

impl Default for FretParams {
    fn default() -> Self {
        Self {
            confidence_threshold: FloatParam::new(
                "Confidence Threshold",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" ")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

impl Default for Fretscope {
    fn default() -> Self {
        Self {
            params: Arc::new(FretParams::default()),
            pitch_detector: pitch::PitchDetector::new(44100.0),
            shared_state: Arc::new(SharedState::default()),
        }
    }
}

impl Plugin for Fretscope {
    const NAME: &'static str = "Fretscope";
    const VENDOR: &'static str = "fretscope";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.shared_state.clone(), self.params.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.pitch_detector = pitch::PitchDetector::new(buffer_config.sample_rate);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Skip detection if not listening
        if !self.shared_state.listening.load(std::sync::atomic::Ordering::Relaxed) {
            return ProcessStatus::Normal;
        }

        let threshold = self.params.confidence_threshold.value();

        // Mix to mono for pitch detection
        let num_samples = buffer.samples();
        let num_channels = buffer.channels();

        let mut mono_samples: Vec<f32> = vec![0.0; num_samples];
        for sample_idx in 0..num_samples {
            let mut sum = 0.0f32;
            for ch in 0..num_channels {
                sum += *buffer.as_slice()[ch].get(sample_idx).unwrap_or(&0.0);
            }
            mono_samples[sample_idx] = sum / num_channels as f32;
        }

        // Feed samples to pitch detector (it accumulates internally)
        if let Some(event) = self.pitch_detector.feed(&mono_samples, threshold) {
            // Compute cents offset for tuner
            let midi_float = 12.0 * (event.frequency / 440.0).log2() + 69.0;
            let cents = (midi_float - midi_float.round()) * 100.0;
            self.shared_state.cents_offset.store(cents, std::sync::atomic::Ordering::Relaxed);

            // Update shared state
            if let Ok(mut note) = self.shared_state.current_note.lock() {
                *note = Some(event.clone());
            }

            // Feed key detector
            if let Ok(mut kd) = self.shared_state.key_detector.lock() {
                kd.add_note(&event);
            }

            // Log note
            if let Ok(mut log) = self.shared_state.note_log.lock() {
                log.push(event);
            }
        }

        // Pass audio through unchanged (this is an analysis plugin)
        ProcessStatus::Normal
    }
}

impl ClapPlugin for Fretscope {
    const CLAP_ID: &'static str = "com.fretscope.fretscope";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Real-time pitch and key detection with guitar fretboard visualization");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Analyzer,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for Fretscope {
    const VST3_CLASS_ID: [u8; 16] = *b"FretScope__xyzab";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Analyzer,
        Vst3SubCategory::Tools,
    ];
}

nih_export_clap!(Fretscope);
nih_export_vst3!(Fretscope);
