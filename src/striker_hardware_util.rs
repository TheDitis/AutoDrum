use serde::{Deserialize, Serialize};
use crate::system_constants::{StrikerConstants, SYSTEM_CONSTANTS};


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
    fn get_constants(striker_kind: StrikerHardwareKind) -> &'static StrikerConstants {
        match striker_kind {
            StrikerHardwareKind::SolenoidBig => &SYSTEM_CONSTANTS.strikers.SolenoidBig,
            StrikerHardwareKind::SolenoidSmall => &SYSTEM_CONSTANTS.strikers.SolenoidSmall,
        }
    }

    pub fn get_min_hit_duration(striker_kind: StrikerHardwareKind) -> f64 {
        StrikerHardwareUtil::get_constants(striker_kind).min_min_hit_duration
    }

    pub fn get_max_hit_duration(striker_kind: StrikerHardwareKind) -> f64 {
        StrikerHardwareUtil::get_constants(striker_kind).max_max_hit_duration
    }

    pub fn get_default_min_hit_duration(striker_kind: StrikerHardwareKind) -> f64 {
        StrikerHardwareUtil::get_constants(striker_kind).default_min_hit_duration
    }

    pub fn get_default_max_hit_duration(striker_kind: StrikerHardwareKind) -> f64 {
        StrikerHardwareUtil::get_constants(striker_kind).default_max_hit_duration
    }
}
