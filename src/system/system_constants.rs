use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use config::{Config, File, FileFormat};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StrikerConstants {
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
pub struct StrikerConstantsMap {
    #[allow(non_snake_case)] // just to match the enum variants
    pub SolenoidBig: StrikerConstants,
    #[allow(non_snake_case)]
    pub SolenoidSmall: StrikerConstants,
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SystemConstants {
    pub strikers: StrikerConstantsMap,
}

lazy_static! {
    pub static ref SYSTEM_CONSTANTS: SystemConstants = {
        // load system-constants.yaml and parse it into a SystemConstants struct
        Config::builder()
            .add_source(File::with_name("system-constants.yaml").required(true))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    };
}

