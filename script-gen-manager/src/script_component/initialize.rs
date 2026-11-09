use std::any::Any;

use script_aggregator::script_buffer::ScriptBuffer;
use script_templates::{error::Result, FunctionTemplate};
use serde::Serialize;

use crate::model::sweep_data::sweep_config::SweepConfig;

use super::function::FunctionModel;

/// [`InitializeModel`] is an aggregation of [`FunctionModel`] that represents the `_Intialize()` function of the script.
/// This is a mandatory function in the generated script.
#[derive(Debug)]
pub struct InitializeModel {
    type_: String,
    description: String,
    template: FunctionTemplate,
}

/// The values referenced by the Initialize template.
#[derive(Debug, Serialize)]
struct InitializeContext {
    max_nodes: u32,
    //TODO: populate from the project's instrument configuration
    product_setup: String,
    append_mode: u32,
    include_timestamps: u32,
    include_srcvals: u32,
}

impl FunctionModel for InitializeModel {
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
        let context = InitializeContext {
            max_nodes: 64,
            product_setup: String::new(),
            append_mode: 1,
            include_timestamps: 1,
            include_srcvals: 1,
        };

        // aux chunk
        if let Some(aux) = self.template.render_aux(&context)? {
            let mut temp = ScriptBuffer::new();
            temp.set_auto_indent(true);

            temp.change_indent(ScriptBuffer::DEFAULT_INDENT);
            for line in aux.lines() {
                temp.body_append(line);
            }
            temp.change_indent(-ScriptBuffer::DEFAULT_INDENT);

            script_buffer.preamble_append(&temp.to_string());
        }

        self.build(script_buffer, &context)
    }
}

impl InitializeModel {
    const DESCRIPTION: &'static str = "This function prepares the test for execution.
    It first verifies that current setup matches project's setup.
    Then, it initializes members used to keep track of reading buffer storage.  ";

    pub fn new(template: FunctionTemplate) -> Self {
        Self {
            type_: template.type_().to_string(),
            description: Self::DESCRIPTION.to_string(),
            template,
        }
    }
}
