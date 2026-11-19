use anyhow::Context;
use std::{fs::File, io::Write, path::Path};

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
    #[must_use]
    pub fn new(catalog: Catalog) -> Self {
        Self {
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
    ///
    /// # Errors
    /// Returns an error if rendering, directory creation, or writing the script fails.
    pub fn to_script(&mut self, sweep_config: &SweepConfig, file_path: &str) -> anyhow::Result<()> {
        let mut script_buffer = ScriptBuffer::new();
        script_buffer.set_auto_indent(true);
        for chunk in &mut self.chunks {
            chunk
                .to_script(sweep_config, &mut script_buffer)
                .with_context(|| format!("Error generating {} script", chunk.get_type()))?;
        }
        //let file_path = "C:\\ScriptGen\\Snippet.txt";
        println!("Writing script to file: {file_path}");
        let path = Path::new(file_path);

        // Check if file exists, if not, create the file and its parent directory if needed
        if !path.exists() {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create directory {}", parent.display())
                    })?;
                }
            }
        }

        let mut file = File::create(file_path)
            .with_context(|| format!("Error creating script file {file_path}"))?;
        file.write_all(script_buffer.to_string().as_bytes())
            .with_context(|| format!("Error writing script file {file_path}"))?;
        file.flush()
            .with_context(|| format!("Error flushing script file {file_path}"))?;
        drop(file);
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use script_templates::FunctionTemplate;

    #[test]
    fn rendering_failure_is_returned_before_touching_the_output() {
        let template = FunctionTemplate::parse(
            "failing.yaml",
            "id: INITIALIZE\ntype: Initialize\nbody: '{{missing_value}}'\n",
        )
        .unwrap();
        let mut model = ScriptModel::new(Catalog::new());
        model.add(Box::new(InitializeModel::new(template)));
        let output =
            std::env::temp_dir().join(format!("script-render-error-{}.tsp", std::process::id()));
        std::fs::write(&output, "existing script").unwrap();

        let error = model
            .to_script(&SweepConfig::new(), output.to_str().unwrap())
            .unwrap_err();
        assert!(format!("{error:#}").contains("Error generating Initialize script"));
        assert_eq!(std::fs::read_to_string(&output).unwrap(), "existing script");
        std::fs::remove_file(output).unwrap();
    }

    #[test]
    fn invalid_parent_directory_returns_a_filesystem_error() {
        let parent =
            std::env::temp_dir().join(format!("script-invalid-parent-{}", std::process::id()));
        std::fs::write(&parent, "not a directory").unwrap();
        let output = parent.join("nested").join("script.tsp");
        let error = ScriptModel::new(Catalog::new())
            .to_script(&SweepConfig::new(), output.to_str().unwrap())
            .unwrap_err();
        assert!(format!("{error:#}").contains("Failed to create directory"));
        std::fs::remove_file(parent).unwrap();
    }
}
