use crate::pitch::NoteEvent;
use std::time::Duration;

const MAX_LOG_SIZE: usize = 200;
const PHRASE_GAP: Duration = Duration::from_millis(500);

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub event: NoteEvent,
    pub is_phrase_start: bool,
}

pub struct NoteLog {
    entries: Vec<LogEntry>,
    last_timestamp: Option<std::time::Instant>,
    last_midi_note: Option<u8>,
}

impl NoteLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(MAX_LOG_SIZE),
            last_timestamp: None,
            last_midi_note: None,
        }
    }

    pub fn push(&mut self, event: NoteEvent) {
        // Skip if same note as last logged
        if self.last_midi_note == Some(event.midi_note) {
            // Still update timestamp so phrase detection works
            self.last_timestamp = Some(event.timestamp);
            return;
        }

        let is_phrase_start = match self.last_timestamp {
            Some(last) => event.timestamp.duration_since(last) > PHRASE_GAP,
            None => true,
        };

        self.last_timestamp = Some(event.timestamp);
        self.last_midi_note = Some(event.midi_note);

        self.entries.push(LogEntry {
            event,
            is_phrase_start,
        });

        if self.entries.len() > MAX_LOG_SIZE {
            self.entries.remove(0);
        }
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.last_timestamp = None;
        self.last_midi_note = None;
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
