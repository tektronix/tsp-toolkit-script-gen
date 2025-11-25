use serde::{Deserialize, Serialize};

use super::sweep_timing_config::SweepTimingConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalParameters {
    pub sweep_timing_config: SweepTimingConfig,
    pub line_frequency: f64,
    pub overhead_time: f64,
}

impl GlobalParameters {
    pub fn new() -> Self {
        GlobalParameters {
            sweep_timing_config: SweepTimingConfig::new(),
            line_frequency: 0.0,
            overhead_time: 78e-6,
        }
    }

    pub fn evaluate(&mut self) {
        self.sweep_timing_config.evaluate();
    }

    pub fn set_line_frequency(&mut self, frequency: f64) {
        self.line_frequency = frequency;
    }
}
