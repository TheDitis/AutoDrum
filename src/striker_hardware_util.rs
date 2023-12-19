use serde::{Deserialize, Serialize};

// Large solenoid constants
const SOLENOID_LG_MIN_HIT_DURATION_MS: f64 = 8.0;
const SOLENOID_LG_MAX_HIT_DURATION_MS: f64 = 75.0;
const SOLENOID_LG_DEFAULT_MIN_HIT_DURATION_MS: f64 = 30.0;
const SOLENOID_LG_DEFAULT_MAX_HIT_DURATION_MS: f64 = 50.0;

// Small solenoid constants
const SOLENOID_SM_MIN_HIT_DURATION_MS: f64 = 0.1;
const SOLENOID_SM_MAX_HIT_DURATION_MS: f64 = 2.0;
const SOLENOID_SM_DEFAULT_MIN_HIT_DURATION_MS: f64 = 0.2;
const SOLENOID_SM_DEFAULT_MAX_HIT_DURATION_MS: f64 = 1.5;


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StrikerHardwareKind {
    SolenoidBig,
    SolenoidSmall,
}

impl TryFrom<&str> for StrikerHardwareKind {
    type Error = String;

    fn try_from(kind: &str) -> Result<Self, Self::Error> {
        match kind {
            "solenoid_big" => Ok(StrikerHardwareKind::SolenoidBig),
            "solenoid_small" => Ok(StrikerHardwareKind::SolenoidSmall),
            _ => Err("Unknown striker hardware kind".to_string()),
        }
    }
}

pub struct StrikerHardwareUtil {}

impl StrikerHardwareUtil {
    pub fn get_min_hit_duration(striker_kind: StrikerHardwareKind) -> f64 {
        match striker_kind {
            StrikerHardwareKind::SolenoidBig => SOLENOID_LG_MIN_HIT_DURATION_MS,
            StrikerHardwareKind::SolenoidSmall => SOLENOID_SM_MIN_HIT_DURATION_MS,
        }
    }

    pub fn get_max_hit_duration(striker_kind: StrikerHardwareKind) -> f64 {
        match striker_kind {
            StrikerHardwareKind::SolenoidBig => SOLENOID_LG_MAX_HIT_DURATION_MS,
            StrikerHardwareKind::SolenoidSmall => SOLENOID_SM_MAX_HIT_DURATION_MS,
        }
    }

    pub fn get_default_min_hit_duration(striker_kind: StrikerHardwareKind) -> f64 {
        match striker_kind {
            StrikerHardwareKind::SolenoidBig => SOLENOID_LG_DEFAULT_MIN_HIT_DURATION_MS,
            StrikerHardwareKind::SolenoidSmall => SOLENOID_SM_DEFAULT_MIN_HIT_DURATION_MS,
        }
    }

    pub fn get_default_max_hit_duration(striker_kind: StrikerHardwareKind) -> f64 {
        match striker_kind {
            StrikerHardwareKind::SolenoidBig => SOLENOID_LG_DEFAULT_MAX_HIT_DURATION_MS,
            StrikerHardwareKind::SolenoidSmall => SOLENOID_SM_DEFAULT_MAX_HIT_DURATION_MS,
        }
    }
}