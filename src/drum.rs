use std::time::Duration;
use std::time::Instant;

use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;
use serde::{Deserialize, Serialize};
use tokio_timerfd::Delay;

use crate::striker::Striker;

const MAX_HIT_DURATION_MS: f64 = 400.0;

/// Represents a drum that can be hit
pub struct Drum {
    /// Human-readable name of the drum (e.g. "Snare" or "Ride Bell)
    pub name: String,
    /// The MIDI note number that triggers this drum
    pub note: u8,
    /// The GPIO pin that controls the striker
    pin: OutputPin,
    /// The type of striker used to hit this drum
    striker: Striker,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DrumRaw {
    pub name: String,
    pub note: u8,
    pub pin: u8,
    pub striker: Striker,
}

impl Drum {
    /// Create a new drum
    pub fn new(note_num: u8, pin_num: u8, name: &str, striker: Striker) -> Self {
        let output_pin = Gpio::new().unwrap().get(pin_num).unwrap().into_output();
        Self {
            name: name.to_string(),
            note: note_num,
            pin: output_pin,
            striker,
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
        let mut duration = self.striker.get_duration(velocity);
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
    pub fn get_striker_kind(&self) -> Striker {
        self.striker
    }

    /// Get the raspberry pi GPIO pin number that controls the striker for this drum
    pub fn get_pin_num(&self) -> u8 { self.pin.pin() }

    pub fn export_raw(&self) -> DrumRaw {
        DrumRaw {
            name: self.name.clone(),
            note: self.note,
            pin: self.pin.pin(),
            striker: self.striker,
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
