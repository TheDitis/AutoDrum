use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::striker_hardware_util::StrikerHardwareKind;

/// A log entry representing a drum hit
#[derive(Debug, Serialize, Deserialize)]
pub struct HitLogEntry {
    /// UNIX timestamp but in milliseconds
    pub time: u64,
    /// milliseconds since the last hit (0 if this is the first hit)
    pub ms_since_last: u64,
    /// nanosecond duration calculated striker duration based on striker type and velocity
    pub planned_duration_ns: u64,
    /// nanosecond duration between when the striker was triggered and when it was turned off
    pub actual_duration_ns: u64,
    /// The type of striker used
    pub striker_kind: StrikerHardwareKind,
    /// The raw MIDI data that triggered the hit
    pub midi_data: (u8, u8, u8),
    /// The note number of the midi data
    pub note_num: u8,
    /// The velocity value of the midi data
    pub velocity: u8,
    /// The name of the drum that was hit
    pub drum_name: String,
    /// The raspberry pi pin number of the drum that was hit
    pub target_pin: u8,
}

pub enum LogEntry {
    /// represents a drum hit triggered by incoming MIDI data
    Hit(HitLogEntry),
}

pub struct Logger {
    /// A stack of all the hits that have been logged
    hit_log: Vec<HitLogEntry>,
}

impl Logger {
    /// Create a new logger
    pub fn new() -> Self {
        Self {
            hit_log: vec![],
        }
    }

    /// Log a given entry to its respective collection
    pub fn log(&mut self, entry: LogEntry) {
        match entry {
            LogEntry::Hit(hit) => self.hit_log.push(hit),
        }
    }

    /// Save the log collections to their respective files
    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.has_data() {
            let mut file = File::create(format!("./logs/hit_log_{:?}.json", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap())).await?;
            file.write_all(serde_json::to_string(&self.hit_log).unwrap().as_bytes()).await?;
            println!("Hit log saved to file");
        }
        Ok(())
    }

    /// Check if any of the collections have data
    pub fn has_data(&self) -> bool {
        !self.hit_log.is_empty()
    }

    /// Get the time of the last hit
    pub fn last_hit_time(&self) -> Option<u64> {
        self.hit_log.last().map(|hit| hit.time)
    }
}