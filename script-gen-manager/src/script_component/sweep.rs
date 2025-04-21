use std::{any::Any, collections::HashMap};

use super::function::FunctionModel;
use script_aggregator::script_buffer::ScriptBuffer;
use xml_handler::group::Group;

/// SweepModel is an aggregation of FunctionModel that represents the _Sweep() function of the script.
/// This function executes a nested step/sweep operation and is optional in the generated script.
#[derive(Debug)]
pub struct SweepModel {
    type_: String,
    description: String,

    metadata: Group,
    val_replacement_map: HashMap<String, String>,
}

impl FunctionModel for SweepModel {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_type(&self) -> &str {
        self.type_.as_str()
    }

    fn get_description(&self) -> &str {
        self.description.as_str()
    }

    fn get_val_replacement_map(&self) -> &std::collections::HashMap<String, String> {
        &self.val_replacement_map
    }

    fn get_metadata(&self) -> &xml_handler::group::Group {
        &self.metadata
    }

    fn to_script(&mut self, script_buffer: &mut ScriptBuffer) {
        // if self.step_channels.is_empty() && self.sweep_channels.is_empty() {
        //     script_buffer.postamble_append(String::from(
        //         "-- no sweep ... requires at least 1 step channel or 1 sweep channel",
        //     ));
        // } else {
        //     self.build(script_buffer);
        // }
    }
}

impl SweepModel {
    const DESCRIPTION: &'static str = "Configures a sweeping test.";
    // const NEGATIVE_NUMBER_NEAR_ZERO: f64 = -1.0e-30;

    pub fn new(group: Group) -> Self {
        SweepModel {
            type_: group.type_.clone(),
            description: Self::DESCRIPTION.to_string(),
            metadata: group,
            val_replacement_map: HashMap::new(),
        }
    }
}
