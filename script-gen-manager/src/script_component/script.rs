use std::{fs::File, io::Write};

use super::{
    data_report::DataReportModel, finalize::FinalizeModel, function::FunctionModel,
    initialize::InitializeModel, sweep::SweepModel,
};
use crate::{catalog::Catalog, model::sweep_data::sweep_config::SweepConfig};
use script_aggregator::script_buffer::ScriptBuffer;

/// Creates and manages the individual functions that make up the script.
pub struct ScriptModel {
    catalog: Catalog,
    chunks: Vec<Box<dyn FunctionModel>>,
}

impl ScriptModel {
    pub fn new(catalog: Catalog) -> Self {
        ScriptModel {
            catalog,
            chunks: Vec::new(), //Initialize with an empty vector
        }
    }

    /// Clears the existing script chunks and adds the initialize and finalize chunks.
    pub fn initialize_scripts(&mut self) {
        self.chunks.clear();

        if let Some(group) = self.catalog.function_metadata_map.get("Initialize") {
            let initialize = InitializeModel::new(group.clone());
            self.chunks.push(Box::new(initialize));
        }

        if let Some(group) = self.catalog.function_metadata_map.get("Finalize") {
            let finalize = FinalizeModel::new(group.clone());
            self.chunks.push(Box::new(finalize));
        }
    }

    /// Converts the script chunks to a script including ordering, indent and substitution.
    pub fn to_script(&mut self, sweep_config: &SweepConfig) {
        let mut script_buffer = ScriptBuffer::new();
        script_buffer.set_auto_indent(true);
        for chunk in self.chunks.iter_mut() {
            chunk.to_script(sweep_config, &mut script_buffer);
        }
        // println!("{}", script_buffer.to_string());
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

    /// Adds a function chunk to the script.
    ///
    /// # Arguments
    ///
    /// * `chunk` - A boxed `FunctionModel` instance to be added to the script.
    pub fn add(&mut self, chunk: Box<dyn FunctionModel>) {
        let index = if self.chunks.len() > 1 {
            self.chunks.len() - 1
        } else {
            0
        };
        self.chunks.insert(index, chunk);
    }

    /// Adds a sweep function chunk to the script.
    pub fn add_sweep(&mut self) {
        if let Some(group) = self.catalog.function_metadata_map.get("Sweep") {
            let sweep = SweepModel::new(group.clone());
            self.add(Box::new(sweep));
        }
    }

    /// Adds a data report function chunk to the script.
    pub fn add_data_report(&mut self) {
        if let Some(group) = self.catalog.function_metadata_map.get("DataReport") {
            let data_report = DataReportModel::new(group.clone());
            self.add(Box::new(data_report));
        }
    }
}
