use serde::{Deserialize, Serialize};

use super::sweep_timing_config::SweepTimingConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalParameters {
    pub sweep_timing_config: SweepTimingConfig,
}

impl Default for GlobalParameters {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalParameters {
    pub fn new() -> Self {
        GlobalParameters {
            sweep_timing_config: SweepTimingConfig::new(),
        }
    }

    pub fn evaluate(&mut self) {
        self.sweep_timing_config.evaluate();
    }
}
