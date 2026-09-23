use serde::{Deserialize, Serialize};
use std::str;

use crate::instr_metadata::base_metadata::BaseMetadata;

use super::parameters::{ParameterFloat, ParameterInt};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StepGlobalParameters {
    pub step_points: ParameterInt,
    pub step_to_sweep_delay: ParameterFloat,
    pub list_step: bool,
}

impl StepGlobalParameters {
    #[must_use]
    pub fn new() -> Self {
        Self {
            step_points: ParameterInt::new("step_points", 10),
            step_to_sweep_delay: ParameterFloat::new(
                "step_to_sweep_delay",
                0.0,
                Some(BaseMetadata::UNIT_SECONDS.to_string()),
            ),
            list_step: false,
        }
    }
}

impl Default for StepGlobalParameters {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SweepGlobalParameters {
    pub sweep_points: ParameterInt,
    pub list_sweep: bool,
}

impl SweepGlobalParameters {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sweep_points: ParameterInt::new("sweep_points", 10),
            list_sweep: false,
        }
    }
}

impl Default for SweepGlobalParameters {
    fn default() -> Self {
        Self::new()
    }
}
