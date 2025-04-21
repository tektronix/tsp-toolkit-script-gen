use serde::{Deserialize, Serialize};

use super::timing_config::TimingConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalParameters {
    pub timing_config: TimingConfig,
}

impl GlobalParameters {
    pub fn new() -> Self {
        GlobalParameters {
            timing_config: TimingConfig::new(),
        }
    }

    pub fn evaluate(&mut self) {
        self.timing_config.evaluate();
    }
}
