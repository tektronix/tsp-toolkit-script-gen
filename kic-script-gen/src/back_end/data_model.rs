use script_gen_manager::model::sweep_data::sweep_model::SweepModel;
use serde_json::json;

use crate::back_end::ipc_data::IpcData;

#[derive(Clone)]
pub struct DataModel {
    pub sweep_model: SweepModel,
}

impl Default for DataModel {
    fn default() -> Self {
        Self::new()
    }
}

impl DataModel {
    pub fn new() -> Self {
        DataModel {
            sweep_model: SweepModel::new(),
        }
    }

    pub fn process_system_config(&mut self, system_info: &str) -> String {
        if self.sweep_model.sweep_config.device_list.is_empty() {
            if self
                .sweep_model
                .sweep_config
                .create_device_list(system_info)
            {
                self.sweep_model.sweep_config.auto_configure();
                self.serialize_sweep_model(
                    &self.sweep_model,
                    "initial_response",
                    "Initialized sweep model",
                )
            } else {
                self.serialize_empty_response("empty_system_config_error", "No devices found")
            }
        } else {
            self.sweep_model
                .sweep_config
                .update_devices_for_changed_slots(system_info);
            self.serialize_sweep_model(
                &self.sweep_model,
                "evaluated_response",
                "Updated sweep model with new system configuration",
            )
        }
    }

    /// Processes data received from the client by deserializing it into a `SweepModel`,
    /// evaluating its configuration, and serializing the result into a response.
    ///
    /// # Arguments
    /// * `data` - A JSON string representing the `SweepModel` to be processed.
    ///
    /// # Returns
    /// A JSON string containing the serialized response. If the deserialization or processing
    /// fails, an error response is returned.
    ///
    /// # Behavior
    /// 1. Attempts to deserialize the input JSON string into a `SweepModel`.
    /// 2. If successful:
    ///    - Evaluates the sweep configuration.
    ///    - Updates the `sweep` field of the `DataModel` with the processed `SweepModel`.
    ///    - Serializes the processed `SweepModel` into a response.
    /// 3. If deserialization fails:
    ///    - Logs the error.
    ///    - Returns an error response.
    pub fn process_data_from_client(&mut self, data: String) -> String {
        // Deserialize the JSON string into a serde_json::Value
        match serde_json::from_str::<SweepModel>(&data) {
            Ok(mut sweep_model) => {
                //println!("Successfully deserialized JSON in server: {sweep_model:?}");
                sweep_model.sweep_config.evaluate();

                //update sweep variable - required for actual script generation
                self.sweep_model = sweep_model.clone();
                self.serialize_sweep_model(
                    &sweep_model,
                    "evaluated_response",
                    "Processed sweep model",
                )
            }
            Err(e) => {
                println!("Failed to deserialize JSON: {e}");
                self.serialize_empty_response("error", "Failed to process sweep model")
            }
        }
    }

    pub fn process_data_from_saved_config(&mut self, data: String) -> String {
        match serde_json::from_str::<SweepModel>(&data) {
            Ok(mut sweep_model) => {
                //println!("Successfully deserialized saved JSON in server: {sweep_model:?}");
                sweep_model.sweep_config.evaluate();

                self.sweep_model = sweep_model.clone();
                self.serialize_sweep_model(
                    &sweep_model,
                    "evaluated_response",
                    "Processed saved sweep model",
                )
            }
            Err(e) => {
                println!("Failed to deserialize saved JSON: {e}");
                self.serialize_empty_response("error", "Failed to process saved sweep model")
            }
        }
    }

    /// Adds, removes, or updates a channel in the `SweepModel` based on the provided `ipc_data`.
    ///
    /// # Arguments
    /// * `ipc_data` - An `IpcData` object containing the JSON representation of the `SweepModel`
    ///   and additional information specifying the operation to perform.
    ///
    /// # Returns
    /// A JSON string containing the serialized response. If the deserialization or processing
    /// fails, an error response is returned.
    ///
    /// # Behavior
    /// 1. Attempts to deserialize the `ipc_data.json_value` into a `SweepModel`.
    /// 2. If successful:
    ///    - Parses the `ipc_data.additional_info` to determine the operation:
    ///      - `"remove"`: Removes a channel specified by the third value in `additional_info`.
    ///      - `"add"`: Adds a channel specified by the second value in `additional_info`.
    ///      - `"update"`: Updates a channel using the second, third, and fourth values in `additional_info`.
    ///    - Updates the `sweep` field of the `DataModel` with the modified `SweepModel`.
    ///    - Serializes the modified `SweepModel` into a response.
    /// 3. If deserialization fails:
    ///    - Logs the error.
    ///    - Returns an error response.
    pub fn add_remove_channel(&mut self, ipc_data: IpcData) -> String {
        match serde_json::from_str::<SweepModel>(ipc_data.json_value.as_str()) {
            Ok(mut sweep_model) => {
                //println!("Successfully deserialized JSON in server: {sweep_model:?}");

                sweep_model.sweep_config.update_channel_devices();
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
                }

                // remove unused and invalid channels
                sweep_model.sweep_config.remove_unused_invalid_channels();

                sweep_model.sweep_config.evaluate();

                self.sweep_model = sweep_model.clone();
                self.serialize_sweep_model(
                    &sweep_model,
                    "evaluated_response",
                    "Processed sweep model",
                )
            }
            Err(e) => {
                println!("Failed to deserialize JSON: {e}");
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
                    println!("Failed to serialize IpcData: {e}");
                    self.serialize_empty_response("error", "Serialization error")
                })
            }
            Err(e) => {
                println!("Failed to serialize sweep model: {e}");
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
            println!("Failed to serialize empty IpcData: {e}");
            "{}".to_string()
        })
    }

    pub fn reset_sweep_config(&mut self) -> String {
        self.sweep_model.sweep_config.reset();
        let sweep_model_clone = self.sweep_model.clone();
        self.serialize_sweep_model(&sweep_model_clone, "reset_response", "Sweep config reset")
    }
}
