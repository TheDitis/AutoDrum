pub enum Striker {
    SolenoidBig,
    SolenoidSmall,
}

impl Striker {
    pub fn get_duration(&self, velocity: u8) -> f64 {
        println!("Getting duration for velocity {}", velocity);
        self.min_hit_duration() + ((velocity as f64 * self.max_hit_duration_variation()) / 127.0)
    }

    fn min_hit_duration(&self) -> f64 {
        (match self {
            Striker::SolenoidBig => 1200,
            Striker::SolenoidSmall => 30,
        }) as f64 / 100.0
    }

    fn max_hit_duration_variation(&self) -> f64 {
        (match self {
            Striker::SolenoidBig => 1000,
            Striker::SolenoidSmall => 200,
        }) as f64 / 100.0
    }
}
