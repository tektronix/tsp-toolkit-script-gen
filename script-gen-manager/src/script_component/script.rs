use crate::{
    catalog::Catalog,
    device::SmuDevice,
    device_manager::{self, DeviceManager},
};

use super::{FunctionModel, InitializeModel};

pub struct ScriptModel {
    device_manager: DeviceManager,
    script_chunks: Vec<Box<dyn FunctionModel>>,
}

impl ScriptModel {
    pub fn new(mut device_manager: DeviceManager) -> Self {
        //parse the xml
        device_manager.catalog.refresh_function_metadata();

        ScriptModel {
            device_manager,
            script_chunks: Vec::new(), //Initialize with an empty vector
        }
    }

    pub fn initialize_scripts(&mut self) {
        self.script_chunks.clear();

        if let Some(group) = self
            .device_manager
            .catalog
            .function_metadata_map
            .get("Initialize")
        {
            let initialize_chunk =
                InitializeModel::new(group.clone(), self.device_manager.device_list.clone());
            self.script_chunks.push(Box::new(initialize_chunk));
        }
    }

    pub fn to_script(&mut self) {
        for chunk in self.script_chunks.iter_mut() {
            chunk.to_script();
        }
    }
}
