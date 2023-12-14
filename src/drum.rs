use rppal::gpio::Gpio;
use std::time::Duration;
use rppal::gpio::OutputPin;
use std::time::Instant;
use tokio_timerfd::Delay;
use crate::striker::Striker;

const MAX_HIT_DURATION_MS: f64 = 400.0;

pub struct Drum {
    note: u8,
    pin: OutputPin,
    striker: Striker,
}


impl Drum {
    pub fn new(note_num: u8, pin_num: u8, striker: Striker) -> Self {
        let output_pin = Gpio::new().unwrap().get(pin_num).unwrap().into_output();
        Self {
            note: note_num,
            pin: output_pin,
            striker,
        }
    }
    pub async fn hit(&mut self, velocity: u8) -> Result<(), std::io::Error> {
        let mut start: tokio::time::Instant;
        if !self.pin.is_set_high() {
            // Get the duration of the hit, clamping if necessary
            let mut duration = self.striker.get_duration(velocity);
            println!("Planned hit duration: {}ms", duration);
            if duration > MAX_HIT_DURATION_MS {
                duration = MAX_HIT_DURATION_MS;
                println!("Clamped hit duration to {}", duration)
            }
            let micros_duration = Duration::from_micros((duration * 1000.0) as u64);
            println!("Micros duration: {:?}", micros_duration);

            // Trigger the striker
            self.pin.set_high();

            // Wait for the duration of the hit, then turn off the striker
            let mut delay = Delay::new(Instant::now() + micros_duration)?;
            delay.await?;
            self.pin.set_low();

            println!("Actual hit duration: {:?}", start.elapsed());
        } else { println!("Drum already hit, ignoring") }
        Ok(())
    }
    pub fn abort(&mut self) {
        self.pin.set_low();
    }
}

impl Drop for Drum {
    fn drop(&mut self) {
        self.pin.set_low();
    }
}
