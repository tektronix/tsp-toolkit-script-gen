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
    pub fn new() -> Self {
        StepGlobalParameters {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SweepGlobalParameters {
    pub sweep_points: ParameterInt,
    pub list_sweep: bool,
}

impl SweepGlobalParameters {
    pub fn new() -> Self {
        SweepGlobalParameters {
            sweep_points: ParameterInt::new("sweep_points", 10),
            list_sweep: false,
        }
    }
}
