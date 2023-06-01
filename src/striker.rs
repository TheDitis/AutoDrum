pub enum Striker {
    SolenoidBig,
    SolenoidSmall,
}

impl Striker {
    pub fn get_duration(&self, velocity: u8) -> u64 {
        println!("Getting duration for velocity {}", velocity);
        self.min_hit_duration() + ((velocity as u64 * self.max_hit_duration_variation()) / 127)
    }

    fn min_hit_duration(&self) -> u64 {
        match self {
            Striker::SolenoidBig => 12,
            Striker::SolenoidSmall => 8,
        }
    }

    fn max_hit_duration_variation(&self) -> u64 {
        match self {
            Striker::SolenoidBig => 10,
            Striker::SolenoidSmall => 10,
        }
    }
}
