use serde::{Deserialize, Serialize};
use std::str;

use crate::instr_metadata::{
    base_metadata::{BaseMetadata, Metadata}, 
    enum_metadata::MetadataEnum
};

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

    // pub fn validate_limits(&mut self, metadata: &MetadataEnum) {
    //     if let Some((min, max)) = Self::get_range_limits(metadata, "source.step_to_sweep_delay") {
    //         self.step_to_sweep_delay.limit(min, max);
    //     }
    // }

    // fn get_range_limits(metadata: &MetadataEnum, key: &str) -> Option<(f64, f64)> {
    //     match metadata {
    //         MetadataEnum::Msmu60(m) => m.get_range(key),
    //         MetadataEnum::Mpsu50(m) => m.get_range(key),
    //         MetadataEnum::Base(m) => m.get_range(key),
    //     }
    // }
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
