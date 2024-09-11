use std::{any::Any, collections::HashMap};

use super::function::FunctionModel;
use script_aggregator::script_buffer::ScriptBuffer;
use xml_handler::group::Group;

#[derive(Debug)]
pub struct DataReportModel {
    type_: String,
    select_mode: SelectMode,
    description: String,
    metadata: Group,
    val_replacement_map: HashMap<String, String>,
}

impl FunctionModel for DataReportModel {
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
        //TODO: Below three replacement values depend on sweep model, will be handled later
        self.val_replacement_map
            .insert(String::from("READING-BUFFERS"), String::from(""));
        self.val_replacement_map
            .insert(String::from("READING-BUFFER-NAMES"), String::from(""));
        self.val_replacement_map
            .insert(String::from("READING-BUFFER-SMU-NAMES"), String::from(""));

        self.val_replacement_map
            .insert(String::from("WAIT-INTERVAL"), String::from("1"));
        self.val_replacement_map
            .insert(String::from("MAX-READINGS-TO-RETURN"), String::from("100"));

        self.val_replacement_map
            .insert(String::from("TAG-DATA-REPORT"), String::from("Data"));
        self.val_replacement_map
            .insert(String::from("TAG-SWEEP-START"), String::from("SWEEP-START"));
        self.val_replacement_map
            .insert(String::from("TAG-START"), String::from("START"));
        self.val_replacement_map.insert(
            String::from("TAG-EXPECTED-COUNT"),
            String::from("EXPECTED-COUNT"),
        );
        self.val_replacement_map
            .insert(String::from("TAG-NAME"), String::from("NAME"));
        self.val_replacement_map
            .insert(String::from("TAG-PTS-IN-BUFF"), String::from("PTS-IN-BUFF"));
        self.val_replacement_map.insert(
            String::from("TAG-PTS-RETURNED"),
            String::from("PTS-RETURNED"),
        );
        self.val_replacement_map.insert(
            String::from("TAG-BASE-TIME-STAMP"),
            String::from("BASE-TIME-STAMP"),
        );
        self.val_replacement_map
            .insert(String::from("TAG-READINGS"), String::from("READINGS"));
        self.val_replacement_map
            .insert(String::from("TAG-TIMESTAMPS"), String::from("TIMESTAMPS"));
        self.val_replacement_map
            .insert(String::from("TAG-SRCVALS"), String::from("SRCVALS"));
        self.val_replacement_map
            .insert(String::from("TAG-END"), String::from("END"));
        self.val_replacement_map
            .insert(String::from("TAG-COMPLETE"), String::from("COMPLETE"));
    }
}

impl DataReportModel {
    const DESCRIPTION: &'static str =
        "The function completes the script and places the instrument in a known state.";

    pub fn new(group: Group) -> Self {
        DataReportModel {
            type_: group.type_.clone(),
            select_mode: SelectMode::Auto,
            description: Self::DESCRIPTION.to_string(),
            metadata: group,
            val_replacement_map: HashMap::new(),
        }
    }
}

#[derive(Debug)]
enum SelectMode {
    Auto,
    Custom,
}
