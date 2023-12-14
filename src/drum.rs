use std::time::Duration;
use std::time::Instant;

use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;
use tokio_timerfd::Delay;

use crate::striker::Striker;

const MAX_HIT_DURATION_MS: f64 = 400.0;

pub struct Drum {
    name: String,
    note: u8,
    pin: OutputPin,
    striker: Striker,
}

impl Drum {
    pub fn new(note_num: u8, pin_num: u8, name: &str, striker: Striker) -> Self {
        let output_pin = Gpio::new().unwrap().get(pin_num).unwrap().into_output();
        Self {
            name: name.to_string(),
            note: note_num,
            pin: output_pin,
            striker,
        }
    }

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

    pub fn abort(&mut self) {
        self.pin.set_low();
    }

    pub fn get_strike_duration(&self, velocity: u8) -> Duration {
        // Get the duration of the hit, clamping if necessary
        let mut duration = self.striker.get_duration(velocity);
        if duration > MAX_HIT_DURATION_MS {
            duration = MAX_HIT_DURATION_MS;
            println!("Clamped hit duration to {}", duration)
        }
        Duration::from_micros((duration * 1000.0) as u64)
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_note_num(&self) -> u8 {
        self.note
    }

    pub fn get_striker_kind(&self) -> Striker {
        self.striker
    }

    pub fn get_pin_num(&self) -> u8 {
        self.pin.pin()
    }
}

impl Drop for Drum {
    fn drop(&mut self) {
        self.pin.set_low();
    }
}
