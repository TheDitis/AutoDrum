use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StrikerConfig {
    // The minimum that the min_hit_duration can be set to
    pub min_min_hit_duration: f64,
    // The maximum that the min_hit_duration can be set to
    pub max_min_hit_duration: f64,
    // The minimum that the max_hit_duration can be set to
    pub min_max_hit_duration: f64,
    // The maximum that the max_hit_duration can be set to
    pub max_max_hit_duration: f64,
    // The default value for min_hit_duration
    pub default_min_hit_duration: f64,
    // The default value for max_hit_duration
    pub default_max_hit_duration: f64,
    // The step size for changing these controls
    pub increment: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StrikerConfigs {
    #[allow(non_snake_case)] // just to match the enum variants
    pub SolenoidBig: StrikerConfig,
    #[allow(non_snake_case)]
    pub SolenoidSmall: StrikerConfig,
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SystemConfig {
    pub strikers: StrikerConfigs,
}

lazy_static! {
    pub static ref SYSTEM_CONFIG: SystemConfig = {
        // load system-config.yaml and parse it into a SystemConfig struct
        let f = std::fs::File::open("system-config.yaml").expect("Unable to open system-config.yaml");
        let config: SystemConfig = serde_yaml::from_reader(f).expect("Unable to parse system-config.yaml");
        config
    };
}

