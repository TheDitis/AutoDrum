use std::thread::sleep;
use rppal::gpio::Gpio;
use std::time::Duration;
use rppal::gpio::OutputPin;
use crate::striker::Striker;


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
    pub async fn hit(&mut self, velocity: u8) {
        let duration = self.striker.get_duration(velocity);
        if !self.pin.is_set_high() {
            println!("[{:?}]: Hitting drum for {}ms", std::time::SystemTime::now(), duration);
            self.pin.set_high();
            tokio::time::sleep(Duration::from_nanos((duration * 1_000.0) as u64)).await;
            // sleep(Duration::from_nanos(duration as u64 * 500_000));
            println!("[{:?}]: Done hitting drum", std::time::SystemTime::now());
        } else { println!("Drum already hit, ignoring") }
        self.pin.set_low();
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
