use config::Config;
use serde::{Deserialize, Serialize};
use crate::hardware::striker::StrikerData;

#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration {
    pub strikers: Vec<StrikerData>
}

impl Configuration {
    pub fn load() -> Self {
        // load system-constants.yaml and parse it into a Configuration struct
        Config::builder()
            .add_source(config::File::with_name("configuration.yaml").required(true))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}
