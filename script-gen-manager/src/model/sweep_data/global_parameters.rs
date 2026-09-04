use serde::{Deserialize, Serialize};

use super::sweep_timing_config::SweepTimingConfig;

const fn default_overhead_time() -> f64 {
    78e-6
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct GlobalParameters {
    pub sweep_timing_config: SweepTimingConfig,
    pub line_frequency: f64,
    #[serde(default = "default_overhead_time")]
    pub overhead_time: f64,
}

impl Default for GlobalParameters {
    fn default() -> Self {
        Self {
            sweep_timing_config: SweepTimingConfig::new(),
            line_frequency: 60.0,
            overhead_time: 78e-6,
        }
    }
}

impl GlobalParameters {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn evaluate(&mut self) {
        self.sweep_timing_config.evaluate();
    }
    pub const fn set_line_frequency(&mut self, frequency: f64) {
        self.line_frequency = frequency;
    }
}
