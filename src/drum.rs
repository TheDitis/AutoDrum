use rppal::gpio::Gpio;
use std::thread::sleep;
use std::time::Duration;
use rppal::gpio::OutputPin;
use crate::striker::Striker;


pub struct Drum {
    pin: OutputPin,
    striker: Striker,
}


impl Drum {
    pub fn new(pin_num: u8) -> Self {
        let output_pin = Gpio::new().unwrap().get(pin_num).unwrap().into_output();
        Self {
            pin: output_pin,
            striker: Striker::SolenoidBig,
        }
    }
    pub async fn hit(&mut self, velocity: u8) {
        let duration = self.striker.get_duration(velocity);
        println!("Hitting drum for {}ms", duration);
        self.pin.set_high();
        tokio::time::sleep(Duration::from_millis((duration as u64) * 1)).await;
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
