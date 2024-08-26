use std::{any::Any, collections::HashMap};

use script_aggregator::script_buffer::ScriptBuffer;
use xml_handler::{composite::Composite, group::Group};

use crate::device::SmuDevice;

use super::FunctionModel;

#[derive(Debug)]
pub struct InitializeModel {
    type_: String,
    description: String,
    metadata: Group,
    device_list: Vec<SmuDevice>,
    val_replacement_map: HashMap<String, String>,
}

impl FunctionModel for InitializeModel {
    fn set_type(&mut self, type_: String) {
        self.type_ = type_;
    }

    fn get_type(&self) -> String {
        self.type_.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_script(&mut self) {
        self.val_replacement_map
            .insert(String::from("MAX-NODES"), String::from("64"));
        self.val_replacement_map
            .insert(String::from("APPEND-MODE"), String::from("1"));
        self.val_replacement_map
            .insert(String::from("INCLUDE-TIMESTAMPS"), String::from("1"));
        self.val_replacement_map
            .insert(String::from("INCLUDE-SRCVALS"), String::from("1"));

        //TODO! aux build stuff

        let mut temp = ScriptBuffer::new();
        temp.set_auto_indent(true);
        for child in self.metadata.children.iter() {
            if let xml_handler::group::IncludeResult::Composite(comp) = child {
                if let Some(_) = comp.type_ {
                    self.aux_build(&mut temp, comp);
                }
            }
        }

        self.val_replacement_map
            .insert(String::from("PRODUCT-SETUP"), self.get_product_setup());
    }
}

impl InitializeModel {
    pub fn new(group: Group, device_list: Vec<SmuDevice>) -> Self {
        InitializeModel {
            type_: group.type_.clone(),
            description: String::from(
                "This function prepares the test for execution. \n 
                 It first verifies that current setup matches project's setup. \n
                 Then, it initializes members used to keep track of reading buffer storage.  ",
            ),
            metadata: group,
            device_list,
            val_replacement_map: HashMap::new(),
        }
    }

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
        for i in 0..unique_nodes.len() {
            let mut formatted_string: String;
            if i == 0 {
                formatted_string = format!(
                    "{{{},[[{}]],[[{}]]}}",
                    unique_nodes[i].get_node_id(),
                    unique_nodes[i].get_model(),
                    unique_nodes[i].get_fw_version()
                );
            } else {
                formatted_string = format!(
                    ",{{{},[[{}]],[[{}]]}}",
                    unique_nodes[i].get_node_id(),
                    unique_nodes[i].get_model(),
                    unique_nodes[i].get_fw_version()
                );
            }
            current_setup.push_str(&formatted_string);
        }
        current_setup.push_str("}");
        println!("{}", current_setup);

        current_setup
    }

    //TODO! aux build needs to be moved to FunctionModel
    //since only InitializeModel needs it, keeping it here for now
    fn aux_build(&self, temp: &mut ScriptBuffer, aux: &Composite) {
        temp.change_indent(ScriptBuffer::DEFAULT_INDENT);
        aux.to_script(temp, &self.val_replacement_map);
    }
}
