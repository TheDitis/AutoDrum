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
        println!("Hitting drum for {}ms", duration);
        self.pin.set_high();
        // tokio::time::sleep(Duration::from_nanos(duration as u64)).await;
        sleep(Duration::from_millis(duration as u64));
        println!("Done hitting drum");
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
