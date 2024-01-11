use std::time::{Duration, Instant};
use rppal::gpio::OutputPin;
use tokio_timerfd::Delay;

pub enum ModifierHardwareKind {
    SolenoidBig
}

pub struct Modifier {
    pub name: String,
    pub note: u8,
    pub pin: OutputPin,
    pub hardware_kind: ModifierHardwareKind,
}

impl Modifier {
    pub fn new(name: &str, note: u8, pin: u8, hardware_kind: ModifierHardwareKind) -> Self {
        let pin = rppal::gpio::Gpio::new().unwrap().get(pin).unwrap().into_output();
        Self {
            name: name.to_string(),
            note,
            pin,
            hardware_kind,
        }
    }

    pub fn activate(&mut self) {
        self.pin.set_high();
    }

    pub fn deactivate(&mut self) {
        self.pin.set_low();
    }

    pub fn is_active(&self) -> bool {
        self.pin.is_set_high()
    }

    pub async fn start_deactivation_timer(&mut self) {
        let delay = Delay::new(
            Instant::now() + Duration::from_millis(self.max_activation_duration() as u64)
        ).unwrap();
        delay.await.unwrap();
        self.deactivate();
    }

    /// Get the maximum duration that the modifier can be activated for in milliseconds
    pub fn max_activation_duration(&self) -> f64 {
        match self.hardware_kind {
            ModifierHardwareKind::SolenoidBig => 5000.0,
        }
    }
}

