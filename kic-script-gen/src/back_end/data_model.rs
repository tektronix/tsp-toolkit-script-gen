use script_gen_manager::{
    catalog::Catalog, device_manager::DeviceManager, model::sweep_data::sweep_model::SweepModel,
};
use serde_json::json;

use crate::back_end::ipc_data::IpcData;

#[derive(Clone)]
pub struct DataModel {
    pub data: String,
    catalog: Catalog,
    pub device_manager: DeviceManager,
}

impl DataModel {
    pub fn new(catalog: Catalog) -> Self {
        DataModel {
            data: String::new(),
            catalog,
            device_manager: DeviceManager::new(),
        }
    }

    pub fn process_instr_info(&mut self, instr_info: String) -> String {
        self.device_manager.create_device_list(instr_info);

        let mut sweep_model = SweepModel::new();
        sweep_model.sweep_config.update_timing_parameters();
        sweep_model.sweep_config.device_list = self.device_manager.device_list.clone();
        sweep_model.sweep_config.auto_configure();

        self.serialize_sweep_model(&sweep_model, "initial_response", "Initialized sweep model")
    }

    pub fn process_data_from_client(&mut self, data: String) -> String {
        // Deserialize the JSON string into a serde_json::Value
        match serde_json::from_str::<SweepModel>(&data) {
            Ok(mut sweep_model) => {
                println!(
                    "Successfully deserialized JSON in server: {:?}",
                    sweep_model
                );
                sweep_model.sweep_config.evaluate();

                self.serialize_sweep_model(
                    &sweep_model,
                    "evaluated_response",
                    "Processed sweep model",
                )
            }
            Err(e) => {
                println!("Failed to deserialize JSON: {}", e);
                self.serialize_empty_response("error", "Failed to process sweep model")
            }
        }
    }

    pub fn add_remove_channel(&mut self, ipc_data: IpcData) -> String {
        match serde_json::from_str::<SweepModel>(ipc_data.json_value.as_str()) {
            Ok(mut sweep_model) => {
                println!(
                    "Successfully deserialized JSON in server: {:?}",
                    sweep_model
                );

                let res: Vec<&str> = ipc_data.additional_info.split(',').collect();
                if res[0] == "remove" {
                    sweep_model.sweep_config.remove_channel(res[2].to_string());
                } else if res[0] == "add" {
                    sweep_model.sweep_config.add_channel(res[1].to_string());
                } else if res[0] == "update" {
                    sweep_model.sweep_config.update_channel(
                        res[1].to_string(),
                        res[2].to_string(),
                        res[3].to_string(),
                    );
                } else {
                    println!("Unknown request type: {}", res[0]);
                }

                self.serialize_sweep_model(
                    &sweep_model,
                    "evaluated_response",
                    "Processed sweep model",
                )
            }
            Err(e) => {
                println!("Failed to deserialize JSON: {}", e);
                self.serialize_empty_response("error", "Failed to process sweep model")
            }
        }
    }

    fn serialize_sweep_model(
        &self,
        sweep_model: &SweepModel,
        request_type: &str,
        additional_info: &str,
    ) -> String {
        match serde_json::to_string(&json!({"sweep_model": sweep_model})) {
            Ok(json_str) => {
                let ipc_data = IpcData {
                    request_type: request_type.to_string(),
                    additional_info: additional_info.to_string(),
                    json_value: json_str,
                };
                serde_json::to_string(&ipc_data).unwrap_or_else(|e| {
                    println!("Failed to serialize IpcData: {}", e);
                    self.serialize_empty_response("error", "Serialization error")
                })
            }
            Err(e) => {
                println!("Failed to serialize sweep model: {}", e);
                self.serialize_empty_response("error", "Serialization error")
            }
        }
    }

    fn serialize_empty_response(&self, request_type: &str, additional_info: &str) -> String {
        let ipc_data = IpcData {
            request_type: request_type.to_string(),
            additional_info: additional_info.to_string(),
            json_value: "{}".to_string(),
        };
        serde_json::to_string(&ipc_data).unwrap_or_else(|e| {
            println!("Failed to serialize empty IpcData: {}", e);
            "{}".to_string()
        })
    }
}
