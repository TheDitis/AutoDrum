use std::time::Duration;
use std::time::Instant;

use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;
use serde::{Deserialize, Serialize};
use tokio_timerfd::Delay;

use crate::striker_hardware_util::{StrikerHardwareKind, StrikerHardwareUtil};


const MAX_HIT_DURATION_MS: f64 = 400.0;

/// Represents a drum that can be hit
pub struct Drum {
    /// Human-readable name of the drum (e.g. "Snare" or "Ride Bell)
    pub name: String,
    /// MIDI note number that triggers this drum
    pub note: u8,
    /// GPIO pin that controls the striker
    pin: OutputPin,
    /// Type of striker used to hit this drum
    striker: StrikerHardwareKind,
    /// Minimum duration of the hit in milliseconds
    min_hit_duration: f64,
    /// Maximum duration of the hit in milliseconds
    max_hit_duration: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DrumRaw {
    pub name: String,
    pub note: u8,
    pub pin: u8,
    pub striker: StrikerHardwareKind,
    pub min_hit_duration: f64,
    pub max_hit_duration: f64,
}

impl Drum {
    /// Create a new drum
    pub fn new(note_num: u8, pin_num: u8, name: &str, striker: StrikerHardwareKind) -> Self {
        let output_pin = Gpio::new().unwrap().get(pin_num).unwrap().into_output();
        Self {
            name: name.to_string(),
            note: note_num,
            pin: output_pin,
            striker,
            min_hit_duration: StrikerHardwareUtil::get_default_min_hit_duration(striker),
            max_hit_duration: StrikerHardwareUtil::get_max_hit_duration(striker),
        }
    }

    /// Hit the drum, triggering the striker for a given duration specified by the striker type and velocity
    pub async fn hit(&mut self, velocity: u8) -> Result<(), std::io::Error> {
        if !self.pin.is_set_high() {
            let duration = self.get_strike_duration(velocity);

            // Trigger the striker
            self.pin.set_high();

            // Wait for the duration of the hit, then turn off the striker
            let delay = Delay::new(Instant::now() + duration)?;
            delay.await?;
            self.pin.set_low();
        } else { println!("Drum already hit, ignoring") }
        Ok(())
    }

    /// Get the duration of the hit based on striker type and velocity, clamping if necessary
    pub fn get_strike_duration(&self, velocity: u8) -> Duration {
        // Get the duration of the hit, clamping if necessary
        let mut duration = self.min_hit_duration + ((velocity as f64 * self.max_hit_duration) / 127.0);
        if duration > MAX_HIT_DURATION_MS {
            duration = MAX_HIT_DURATION_MS;
            println!("Clamped hit duration to {}", duration)
        }
        Duration::from_micros((duration * 1000.0) as u64)
    }

    /// Get the name of the drum
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Get the MIDI note number of the drum
    pub fn get_note_num(&self) -> u8 {
        self.note
    }

    /// Get the type of striker used to hit the drum
    pub fn get_striker_kind(&self) -> StrikerHardwareKind {
        self.striker
    }

    /// Get the raspberry pi GPIO pin number that controls the striker for this drum
    pub fn get_pin_num(&self) -> u8 { self.pin.pin() }

    /// Get the minimum duration of the hit in milliseconds
    pub fn get_min_hit_duration(&self) -> f64 { self.min_hit_duration }

    /// Get the maximum duration of the hit in milliseconds
    pub fn get_max_hit_duration(&self) -> f64 { self.max_hit_duration }

    pub fn export_raw(&self) -> DrumRaw {
        DrumRaw {
            name: self.name.clone(),
            note: self.note,
            pin: self.pin.pin(),
            striker: self.striker,
            min_hit_duration: self.min_hit_duration,
            max_hit_duration: self.max_hit_duration,
        }
    }

    /// Abort the current hit, turning off the striker early
    pub fn abort(&mut self) {
        self.pin.set_low();
    }
}

/// Automatically turn off the striker when the drum is dropped (e.g. there's a panic during a hit)
impl Drop for Drum {
    fn drop(&mut self) {
        self.pin.set_low();
    }
}
