use std::{any::Any, collections::HashMap};

use super::function::FunctionModel;
use crate::model::sweep_data::sweep_config::SweepConfig;
use script_aggregator::script_buffer::ScriptBuffer;
use xml_handler::group::Group;

/// FinalizeModel is an aggregation of FunctionModel that represents the _Finalize() function of the script.
/// This is a mandatory function in the generated script.
#[derive(Debug)]
pub struct FinalizeModel {
    type_: String,
    description: String,
    metadata: Group,
    val_replacement_map: HashMap<String, String>,
}

impl FunctionModel for FinalizeModel {
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

    fn to_script(&mut self, sweep_config: &SweepConfig, script_buffer: &mut ScriptBuffer) {
        self.build(script_buffer);
    }
}

impl FinalizeModel {
    const DESCRIPTION: &'static str =
        "The function completes the script and places the instrument in a known state.";

    pub fn new(group: Group) -> Self {
        FinalizeModel {
            type_: group.type_.clone(),
            description: Self::DESCRIPTION.to_string(),
            metadata: group,
            val_replacement_map: HashMap::new(),
        }
    }
}
