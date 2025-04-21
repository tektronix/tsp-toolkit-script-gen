use std::{any::Any, collections::HashMap};

use script_aggregator::script_buffer::ScriptBuffer;
use xml_handler::{composite::CommonChunk, group::Group};

use super::function::FunctionModel;

/// InitializeModel is an aggregation of FunctionModel that represents the _Intialize() function of the script.
/// This is a mandatory function in the generated script.
#[derive(Debug)]
pub struct InitializeModel {
    type_: String,
    description: String,
    metadata: Group,
    val_replacement_map: HashMap<String, String>,
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

    fn get_val_replacement_map(&self) -> &std::collections::HashMap<String, String> {
        &self.val_replacement_map
    }

    fn get_metadata(&self) -> &xml_handler::group::Group {
        &self.metadata
    }

    fn to_script(&mut self, script_buffer: &mut ScriptBuffer) {
        self.val_replacement_map
            .insert(String::from("MAX-NODES"), String::from("64"));
        self.val_replacement_map
            .insert(String::from("APPEND-MODE"), String::from("1"));
        self.val_replacement_map
            .insert(String::from("INCLUDE-TIMESTAMPS"), String::from("1"));
        self.val_replacement_map
            .insert(String::from("INCLUDE-SRCVALS"), String::from("1"));

        for child in self.metadata.children.iter_mut() {
            if let xml_handler::group::IncludeResult::Composite(comp) = child {
                // aux chunk
                if comp.type_.is_some() {
                    let mut temp = ScriptBuffer::new();
                    temp.set_auto_indent(true);

                    temp.change_indent(ScriptBuffer::DEFAULT_INDENT);
                    comp.to_script(&mut temp, &self.val_replacement_map);
                    temp.change_indent(-ScriptBuffer::DEFAULT_INDENT);

                    script_buffer.preamble_append(temp.to_string());
                }
            }
        }

        // self.val_replacement_map
        //     .insert(String::from("PRODUCT-SETUP"), self.get_product_setup());

        self.build(script_buffer);
    }
}

impl InitializeModel {
    const DESCRIPTION: &'static str = "This function prepares the test for execution.
    It first verifies that current setup matches project's setup.
    Then, it initializes members used to keep track of reading buffer storage.  ";

    pub fn new(group: Group) -> Self {
        InitializeModel {
            type_: group.type_.clone(),
            description: Self::DESCRIPTION.to_string(),
            metadata: group,
            val_replacement_map: HashMap::new(),
        }
    }
}
