use std::{any::Any, collections::HashMap};

use super::function::FunctionModel;
use script_aggregator::script_buffer::ScriptBuffer;
use xml_handler::group::Group;

/// DataReportModel is an aggregation of FunctionModel that represents the _DataReport() function of the script.
/// This is an optional function in the generated script.
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
        let buffers = String::from("{}");
        let buffer_names = String::from("{}");
        let buffer_smu_names = String::from("{}");

        self.val_replacement_map
            .insert(String::from("READING-BUFFERS"), buffers);
        self.val_replacement_map
            .insert(String::from("READING-BUFFER-NAMES"), buffer_names);
        self.val_replacement_map
            .insert(String::from("READING-BUFFER-SMU-NAMES"), buffer_smu_names);

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

        self.build(script_buffer);
    }
}

impl DataReportModel {
    const DESCRIPTION: &'static str = "This script returns a series of reading buffers. ";

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

#[derive(Debug, Clone, PartialEq)]
enum SelectMode {
    Auto,
    Custom,
}
