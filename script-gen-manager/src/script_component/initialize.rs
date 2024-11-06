use std::{any::Any, collections::HashMap};

use script_aggregator::script_buffer::ScriptBuffer;
use xml_handler::{composite::CommonChunk, group::Group};

use super::function::FunctionModel;
use crate::device::SmuDevice;

/// InitializeModel is an aggregation of FunctionModel that represents the _Intialize() function of the script.
/// This is a mandatory function in the generated script.
#[derive(Debug)]
pub struct InitializeModel {
    type_: String,
    description: String,
    metadata: Group,
    val_replacement_map: HashMap<String, String>,

    device_list: Vec<SmuDevice>,
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

        self.val_replacement_map
            .insert(String::from("PRODUCT-SETUP"), self.get_product_setup());

        self.build(script_buffer);
    }
}

impl InitializeModel {
    const DESCRIPTION: &'static str = "This function prepares the test for execution.
    It first verifies that current setup matches project's setup.
    Then, it initializes members used to keep track of reading buffer storage.  ";

    pub fn new(group: Group, device_list: Vec<SmuDevice>) -> Self {
        InitializeModel {
            type_: group.type_.clone(),
            description: Self::DESCRIPTION.to_string(),
            metadata: group,
            val_replacement_map: HashMap::new(),
            device_list,
        }
    }

    /// Generates the product setup string based on the unique nodes in the device list.
    /// e.g., node[37].smua and node[37].smub are considered as a single node.
    ///
    /// # Returns
    ///
    /// A string representing the product setup.
    fn get_product_setup(&self) -> String {
        let mut unique_nodes: Vec<SmuDevice> = Vec::new();

        for device in self.device_list.iter() {
            let mut node_found = false;

            if unique_nodes.is_empty() {
                unique_nodes.push(device.clone());
            } else {
                for node in unique_nodes.iter() {
                    if node.get_node_id() == device.get_node_id() {
                        node_found = true;
                        break;
                    }
                }

                if !node_found {
                    unique_nodes.push(device.clone());
                }
            }
        }

        let mut current_setup = String::from("{");
        for (i, node) in unique_nodes.iter().enumerate() {
            let formatted_string: String = if i == 0 {
                format!(
                    "{{{},[[{}]],[[{}]]}}",
                    node.get_node_id(),
                    node.get_model(),
                    node.get_fw_version()
                )
            } else {
                format!(
                    ",{{{},[[{}]],[[{}]]}}",
                    node.get_node_id(),
                    node.get_model(),
                    node.get_fw_version()
                )
            };
            current_setup.push_str(&formatted_string);
        }
        current_setup.push('}');

        current_setup
    }
}
