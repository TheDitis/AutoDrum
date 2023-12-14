use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Striker {
    SolenoidBig,
    SolenoidSmall,
}

impl Striker {
    pub fn get_duration(&self, velocity: u8) -> f64 {
        self.min_hit_duration() + ((velocity as f64 * self.max_hit_duration_variation()) / 127.0)
    }

    fn min_hit_duration(&self) -> f64 {
        match self {
            Striker::SolenoidBig => 10.0,
            Striker::SolenoidSmall => 0.2,
        }
    }

    fn max_hit_duration_variation(&self) -> f64 {
        match self {
            Striker::SolenoidBig => 20.0,
            Striker::SolenoidSmall => 1.5,
        }
    }
}
