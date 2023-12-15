use serde::{Deserialize, Serialize};

/// Represents a striker that can hit a drum
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Striker {
    /// A large solenoid that can hit a larger drum harder. Has a longer hit duration and a larger hit duration variation
    SolenoidBig,
    /// A small solenoid that can hit a smaller drum softer. Has a shorter hit duration and a smaller hit duration variation
    SolenoidSmall,
}

impl Striker {
    /// Get the ideal duration of the hit in milliseconds based on striker type and velocity
    pub fn get_duration(&self, velocity: u8) -> f64 {
        self.min_hit_duration() + ((velocity as f64 * self.max_hit_duration_variation()) / 127.0)
    }

    /// Get the minimum hit duration for this striker type in milliseconds.
    /// This is the shortest time it can be activated while still getting a sound
    fn min_hit_duration(&self) -> f64 {
        match self {
            Striker::SolenoidBig => 10.0,
            Striker::SolenoidSmall => 0.2,
        }
    }

    /// Get the maximum hit duration variation for this striker type in milliseconds.
    /// The maximum duration is min_hit_duration + (max_hit_duration_variation * (velocity / 127))
    fn max_hit_duration_variation(&self) -> f64 {
        match self {
            Striker::SolenoidBig => 20.0,
            Striker::SolenoidSmall => 1.5,
        }
    }
}
