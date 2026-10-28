use std::any::Any;

use super::function::FunctionModel;
use crate::model::sweep_data::sweep_config::SweepConfig;
use script_aggregator::script_buffer::ScriptBuffer;
use script_templates::{error::Result, FunctionTemplate};

/// [`FinalizeModel`] is an aggregation of [`FunctionModel`] that represents the `_Finalize()` function of the script.
/// This is a mandatory function in the generated script.
#[derive(Debug)]
pub struct FinalizeModel {
    type_: String,
    description: String,
    template: FunctionTemplate,
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

    fn get_template(&self) -> &FunctionTemplate {
        &self.template
    }

    fn to_script(
        &mut self,
        sweep_config: &SweepConfig,
        script_buffer: &mut ScriptBuffer,
    ) -> Result<()> {
        self.build(script_buffer, &())
    }
}

impl FinalizeModel {
    const DESCRIPTION: &'static str =
        "The function completes the script and places the instrument in a known state.";

    pub fn new(template: FunctionTemplate) -> Self {
        Self {
            type_: template.type_().to_string(),
            description: Self::DESCRIPTION.to_string(),
            template,
        }
    }
}
