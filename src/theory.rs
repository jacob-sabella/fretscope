/// Music theory engine — scales, modes, chord voicings, and fretboard mapping.
///
/// All interval data is defined as semitone offsets from the root (pitch class 0-11).
/// This is sourced from standard music theory definitions to ensure 100% accuracy.

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

const MAX_FRETS: u8 = 30;

/// Tuning preset: name + MIDI note numbers (low to high)
#[derive(Debug, Clone)]
pub struct Tuning {
    pub name: String,
    pub notes: Vec<u8>,
}

impl Tuning {
    pub fn string_count(&self) -> usize {
        self.notes.len()
    }

    pub fn note_label(&self, string_idx: usize) -> String {
        if string_idx < self.notes.len() {
            let midi = self.notes[string_idx];
            let pc = midi % 12;
            let octave = (midi as i32 / 12) - 1;
            format!("{}{}", NOTE_NAMES[pc as usize], octave)
        } else {
            "?".to_string()
        }
    }
}

pub fn preset_tunings() -> Vec<Tuning> {
    vec![
        Tuning { name: "Standard (EADGBE)".into(), notes: vec![40, 45, 50, 55, 59, 64] },
        Tuning { name: "Drop D (DADGBE)".into(), notes: vec![38, 45, 50, 55, 59, 64] },
        Tuning { name: "Drop C (CADGBE)".into(), notes: vec![36, 43, 48, 53, 57, 62] },
        Tuning { name: "Open G (DGDGBD)".into(), notes: vec![38, 43, 50, 55, 59, 62] },
        Tuning { name: "Open D (DADF#AD)".into(), notes: vec![38, 45, 50, 54, 57, 62] },
        Tuning { name: "DADGAD".into(), notes: vec![38, 45, 50, 55, 57, 62] },
        Tuning { name: "Half Step Down".into(), notes: vec![39, 44, 49, 54, 58, 63] },
        Tuning { name: "Full Step Down".into(), notes: vec![38, 43, 48, 53, 57, 62] },
        Tuning { name: "7-String Standard".into(), notes: vec![35, 40, 45, 50, 55, 59, 64] },
        Tuning { name: "7-String Drop A".into(), notes: vec![33, 40, 45, 50, 55, 59, 64] },
        Tuning { name: "8-String Standard".into(), notes: vec![30, 35, 40, 45, 50, 55, 59, 64] },
        Tuning { name: "Bass 4-String (EADG)".into(), notes: vec![28, 33, 38, 43] },
        Tuning { name: "Bass 5-String (BEADG)".into(), notes: vec![23, 28, 33, 38, 43] },
    ]
}

/// All note names with octaves for MIDI range (useful for custom tuning picker)
pub fn midi_to_label(midi: u8) -> String {
    let pc = midi % 12;
    let octave = (midi as i32 / 12) - 1;
    format!("{}{}", NOTE_NAMES[pc as usize], octave)
}

// ── Scale definitions (semitone intervals from root) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleType {
    Major,
    NaturalMinor,
    HarmonicMinor,
    MelodicMinor,
    PentatonicMajor,
    PentatonicMinor,
    Blues,
    WholeTone,
    Diminished,
    // Modes of the major scale
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,
    Locrian,
}

impl ScaleType {
    pub fn intervals(&self) -> &'static [u8] {
        match self {
            ScaleType::Major => &[0, 2, 4, 5, 7, 9, 11],
            ScaleType::NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
            ScaleType::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            ScaleType::MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
            ScaleType::PentatonicMajor => &[0, 2, 4, 7, 9],
            ScaleType::PentatonicMinor => &[0, 3, 5, 7, 10],
            ScaleType::Blues => &[0, 3, 5, 6, 7, 10],
            ScaleType::WholeTone => &[0, 2, 4, 6, 8, 10],
            ScaleType::Diminished => &[0, 2, 3, 5, 6, 8, 9, 11],
            ScaleType::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            ScaleType::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            ScaleType::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            ScaleType::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            ScaleType::Aeolian => &[0, 2, 3, 5, 7, 8, 10],
            ScaleType::Locrian => &[0, 1, 3, 5, 6, 8, 10],
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ScaleType::Major => "Major (Ionian)",
            ScaleType::NaturalMinor => "Natural Minor",
            ScaleType::HarmonicMinor => "Harmonic Minor",
            ScaleType::MelodicMinor => "Melodic Minor",
            ScaleType::PentatonicMajor => "Pentatonic Major",
            ScaleType::PentatonicMinor => "Pentatonic Minor",
            ScaleType::Blues => "Blues",
            ScaleType::WholeTone => "Whole Tone",
            ScaleType::Diminished => "Diminished",
            ScaleType::Dorian => "Dorian",
            ScaleType::Phrygian => "Phrygian",
            ScaleType::Lydian => "Lydian",
            ScaleType::Mixolydian => "Mixolydian",
            ScaleType::Aeolian => "Aeolian",
            ScaleType::Locrian => "Locrian",
        }
    }

    pub const ALL: &'static [ScaleType] = &[
        ScaleType::Major,
        ScaleType::NaturalMinor,
        ScaleType::HarmonicMinor,
        ScaleType::MelodicMinor,
        ScaleType::PentatonicMajor,
        ScaleType::PentatonicMinor,
        ScaleType::Blues,
        ScaleType::WholeTone,
        ScaleType::Diminished,
        ScaleType::Dorian,
        ScaleType::Phrygian,
        ScaleType::Lydian,
        ScaleType::Mixolydian,
        ScaleType::Aeolian,
        ScaleType::Locrian,
    ];
}

// ── Chord definitions ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordType {
    Major,
    Minor,
    Dominant7,
    Major7,
    Minor7,
    Diminished,
    Augmented,
    Sus2,
    Sus4,
    Add9,
    Power,
}

impl ChordType {
    pub fn intervals(&self) -> &'static [u8] {
        match self {
            ChordType::Major => &[0, 4, 7],
            ChordType::Minor => &[0, 3, 7],
            ChordType::Dominant7 => &[0, 4, 7, 10],
            ChordType::Major7 => &[0, 4, 7, 11],
            ChordType::Minor7 => &[0, 3, 7, 10],
            ChordType::Diminished => &[0, 3, 6],
            ChordType::Augmented => &[0, 4, 8],
            ChordType::Sus2 => &[0, 2, 7],
            ChordType::Sus4 => &[0, 5, 7],
            ChordType::Add9 => &[0, 2, 4, 7],
            ChordType::Power => &[0, 7],
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            ChordType::Major => "",
            ChordType::Minor => "m",
            ChordType::Dominant7 => "7",
            ChordType::Major7 => "maj7",
            ChordType::Minor7 => "m7",
            ChordType::Diminished => "dim",
            ChordType::Augmented => "aug",
            ChordType::Sus2 => "sus2",
            ChordType::Sus4 => "sus4",
            ChordType::Add9 => "add9",
            ChordType::Power => "5",
        }
    }

    pub const ALL: &'static [ChordType] = &[
        ChordType::Major,
        ChordType::Minor,
        ChordType::Dominant7,
        ChordType::Major7,
        ChordType::Minor7,
        ChordType::Diminished,
        ChordType::Augmented,
        ChordType::Sus2,
        ChordType::Sus4,
        ChordType::Add9,
        ChordType::Power,
    ];
}

// ── Fretboard position mapping ──

#[derive(Debug, Clone)]
pub struct FretPosition {
    /// Guitar string (0 = lowest, N = highest)
    pub string: u8,
    /// Fret number (0 = open)
    pub fret: u8,
    /// MIDI note number
    pub midi_note: u8,
    /// Note name
    pub note_name: String,
    /// Whether this is the root of the scale/chord
    pub is_root: bool,
    /// Interval from root (semitones)
    pub interval: u8,
    /// Scale degree (1-based index in the scale, 0 if not in scale)
    pub scale_degree: u8,
}

/// Get the scale degree label for a given degree number (1-based)
pub fn degree_label(degree: u8) -> &'static str {
    match degree {
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        _ => "?",
    }
}

/// Get note name for a MIDI note number
pub fn note_name(midi: u8) -> &'static str {
    NOTE_NAMES[(midi % 12) as usize]
}

/// Get all note names
pub fn all_note_names() -> &'static [&'static str; 12] {
    &NOTE_NAMES
}

/// Map a scale onto the fretboard, returning all positions where scale tones appear.
pub fn scale_positions(root_pc: u8, scale: ScaleType, tuning: &Tuning, num_frets: u8) -> Vec<FretPosition> {
    let intervals = scale.intervals();
    let mut positions = Vec::new();

    for (string_idx, &open_note) in tuning.notes.iter().enumerate() {
        for fret in 0..=num_frets.min(MAX_FRETS) {
            let midi = open_note + fret;
            let pc = midi % 12;
            let interval_from_root = (pc + 12 - root_pc) % 12;

            if intervals.contains(&interval_from_root) {
                // Compute scale degree (1-based position in the interval list)
                let degree = intervals
                    .iter()
                    .position(|&i| i == interval_from_root)
                    .map(|d| (d + 1) as u8)
                    .unwrap_or(0);

                positions.push(FretPosition {
                    string: string_idx as u8,
                    fret,
                    midi_note: midi,
                    note_name: NOTE_NAMES[pc as usize].to_string(),
                    is_root: interval_from_root == 0,
                    interval: interval_from_root,
                    scale_degree: degree,
                });
            }
        }
    }

    positions
}

/// Map a chord onto the fretboard.
pub fn chord_positions(root_pc: u8, chord: ChordType, tuning: &Tuning, num_frets: u8) -> Vec<FretPosition> {
    let intervals = chord.intervals();
    let mut positions = Vec::new();

    for (string_idx, &open_note) in tuning.notes.iter().enumerate() {
        for fret in 0..=num_frets.min(MAX_FRETS) {
            let midi = open_note + fret;
            let pc = midi % 12;
            let interval_from_root = (pc + 12 - root_pc) % 12;

            if intervals.contains(&interval_from_root) {
                let degree = intervals
                    .iter()
                    .position(|&i| i == interval_from_root)
                    .map(|d| (d + 1) as u8)
                    .unwrap_or(0);

                positions.push(FretPosition {
                    string: string_idx as u8,
                    fret,
                    midi_note: midi,
                    note_name: NOTE_NAMES[pc as usize].to_string(),
                    is_root: interval_from_root == 0,
                    interval: interval_from_root,
                    scale_degree: degree,
                });
            }
        }
    }

    positions
}

/// Common open/barre chord voicings for guitar.
/// Each voicing is (string, fret) pairs for 6 strings.
/// -1 means string is muted/not played.
#[derive(Debug, Clone)]
pub struct ChordVoicing {
    pub name: String,
    pub chord_type: ChordType,
    /// Fret for each string (low E to high E). -1 = muted.
    pub frets: [i8; 6],
    /// Root note pitch class
    pub root_pc: u8,
}

/// Get common voicings for a given root and chord type.
/// These are standard guitar chord shapes transposed to the requested root.
pub fn common_voicings(root_pc: u8, chord_type: ChordType) -> Vec<ChordVoicing> {
    let root_name = NOTE_NAMES[root_pc as usize];
    let suffix = chord_type.suffix();

    // Base shapes defined relative to open position for specific roots,
    // then we compute transpositions via barre chords.
    let mut voicings = Vec::new();

    // E-shape barre chord (root on 6th string)
    let e_string_root_fret = (root_pc as i8 - 4 + 12) % 12; // E is pitch class 4
    match chord_type {
        ChordType::Major => {
            let f = e_string_root_fret;
            voicings.push(ChordVoicing {
                name: format!("{}{} (E shape)", root_name, suffix),
                chord_type,
                frets: [f, f + 2, f + 2, f + 1, f, f],
                root_pc,
            });
        }
        ChordType::Minor => {
            let f = e_string_root_fret;
            voicings.push(ChordVoicing {
                name: format!("{}{} (E shape)", root_name, suffix),
                chord_type,
                frets: [f, f + 2, f + 2, f, f, f],
                root_pc,
            });
        }
        _ => {}
    }

    // A-shape barre chord (root on 5th string)
    let a_string_root_fret = (root_pc as i8 - 9 + 12) % 12; // A is pitch class 9
    match chord_type {
        ChordType::Major => {
            let f = a_string_root_fret;
            voicings.push(ChordVoicing {
                name: format!("{}{} (A shape)", root_name, suffix),
                chord_type,
                frets: [-1, f, f + 2, f + 2, f + 2, f],
                root_pc,
            });
        }
        ChordType::Minor => {
            let f = a_string_root_fret;
            voicings.push(ChordVoicing {
                name: format!("{}{} (A shape)", root_name, suffix),
                chord_type,
                frets: [-1, f, f + 2, f + 2, f + 1, f],
                root_pc,
            });
        }
        ChordType::Power => {
            let f = a_string_root_fret;
            voicings.push(ChordVoicing {
                name: format!("{}{} (A shape)", root_name, suffix),
                chord_type,
                frets: [-1, f, f + 2, f + 2, -1, -1],
                root_pc,
            });
        }
        _ => {}
    }

    // E-shape power chord (root on 6th string)
    if chord_type == ChordType::Power {
        let f = e_string_root_fret;
        voicings.push(ChordVoicing {
            name: format!("{}{} (E shape)", root_name, suffix),
            chord_type,
            frets: [f, f + 2, f + 2, -1, -1, -1],
            root_pc,
        });
    }

    voicings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_major_scale_intervals() {
        let intervals = ScaleType::Major.intervals();
        assert_eq!(intervals, &[0, 2, 4, 5, 7, 9, 11]);
    }

    #[test]
    fn test_scale_positions_contain_root() {
        let std_tuning = &preset_tunings()[0]; // Standard EADGBE
        let positions = scale_positions(0, ScaleType::Major, std_tuning, 24); // C Major
        let roots: Vec<_> = positions.iter().filter(|p| p.is_root).collect();
        assert!(!roots.is_empty());
        assert!(roots.iter().all(|p| p.note_name == "C"));
    }

    #[test]
    fn test_a_minor_pentatonic() {
        let intervals = ScaleType::PentatonicMinor.intervals();
        assert_eq!(intervals, &[0, 3, 5, 7, 10]);
    }
}
