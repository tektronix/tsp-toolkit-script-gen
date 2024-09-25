use std::{fs::File, io::Write};

use super::{
    data_report::DataReportModel, finalize::FinalizeModel, function::FunctionModel,
    initialize::InitializeModel, sweep::SweepModel,
};
use crate::device_manager::DeviceManager;
use script_aggregator::script_buffer::ScriptBuffer;

pub struct ScriptModel {
    device_manager: DeviceManager,
    chunks: Vec<Box<dyn FunctionModel>>,
}

impl ScriptModel {
    pub fn new(mut device_manager: DeviceManager) -> Self {
        //parse the xml
        device_manager.catalog.refresh_function_metadata();

        ScriptModel {
            device_manager,
            chunks: Vec::new(), //Initialize with an empty vector
        }
    }

    pub fn initialize_scripts(&mut self) {
        self.chunks.clear();

        if let Some(group) = self
            .device_manager
            .catalog
            .function_metadata_map
            .get("Initialize")
        {
            let initialize =
                InitializeModel::new(group.clone(), self.device_manager.device_list.clone());
            self.chunks.push(Box::new(initialize));
        }

        if let Some(group) = self
            .device_manager
            .catalog
            .function_metadata_map
            .get("Finalize")
        {
            let finalize = FinalizeModel::new(group.clone());
            self.chunks.push(Box::new(finalize));
        }
    }

    pub fn to_script(&mut self) {
        let mut script_buffer = ScriptBuffer::new();
        script_buffer.set_auto_indent(true);
        for chunk in self.chunks.iter_mut() {
            chunk.to_script(&mut script_buffer);
        }
        println!("{}", script_buffer.to_string());
        let file = File::create("C:\\Trebuchet\\Snippet.txt");
        match file {
            Ok(mut file_res) => {
                file_res.write_all(script_buffer.to_string().as_bytes());
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    pub fn add(&mut self, chunk: Box<dyn FunctionModel>) {
        let index = if self.chunks.len() > 1 {
            self.chunks.len() - 1
        } else {
            0
        };
        self.chunks.insert(index, chunk);
    }

    pub fn add_sweep(&mut self) {
        if let Some(group) = self
            .device_manager
            .catalog
            .function_metadata_map
            .get("Sweep")
        {
            let mut sweep = SweepModel::new(group.clone());
            sweep.auto_configure(&self.device_manager);
            self.add(Box::new(sweep));
        }
    }

    pub fn add_data_report(&mut self) {
        if let Some(group) = self
            .device_manager
            .catalog
            .function_metadata_map
            .get("DataReport")
        {
            let data_report = DataReportModel::new(group.clone());
            self.add(Box::new(data_report));
        }
    }
}
