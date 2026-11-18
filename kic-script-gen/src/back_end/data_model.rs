use script_gen_manager::model::sweep_data::{
    status_msg::{StatusMsg, StatusType},
    sweep_model::SweepModel,
};
use serde::Serialize;

use crate::back_end::ipc_data::IpcData;

#[derive(Clone)]
pub struct DataModel {
    pub sweep_model: SweepModel,
}

#[derive(Serialize)]
struct SweepModelEnvelope<'a> {
    sweep_model: &'a SweepModel,
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
                self.sweep_model = sweep_model;
                self.serialize_sweep_model(
                    &self.sweep_model,
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

                self.sweep_model = sweep_model;
                self.serialize_sweep_model(
                    &self.sweep_model,
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
                let accepted = match res.as_slice() {
                    ["remove", "bias" | "step" | "sweep", address, model] => sweep_model
                        .sweep_config
                        .remove_channel_for_model(address, model),
                    ["add", role @ ("bias" | "step" | "sweep")] => {
                        sweep_model.sweep_config.add_channel(role);
                        true
                    }
                    ["update", role, old_address, new_address, old_model, new_model] => {
                        sweep_model.sweep_config.update_channel_for_models(
                            role,
                            old_address,
                            new_address,
                            old_model,
                            new_model,
                        )
                    }
                    _ => {
                        println!("Invalid channel operation: {}", ipc_data.additional_info);
                        sweep_model.sweep_config.status_msg = Some(StatusMsg::new(
                            StatusType::Error,
                            String::from(
                                "Invalid channel operation. Existing configuration preserved.",
                            ),
                            Some(0),
                        ));
                        false
                    }
                };
                if !accepted {
                    self.sweep_model.sweep_config.status_msg =
                        sweep_model.sweep_config.status_msg.clone();
                    return self.serialize_sweep_model(
                        &self.sweep_model,
                        "evaluated_response",
                        "Channel operation rejected; existing configuration preserved",
                    );
                }

                // remove unused and invalid channels
                sweep_model.sweep_config.remove_unused_invalid_channels();

                sweep_model.sweep_config.evaluate();

                self.sweep_model = sweep_model;
                self.serialize_sweep_model(
                    &self.sweep_model,
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
        match serde_json::to_string(&SweepModelEnvelope { sweep_model }) {
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
        self.serialize_sweep_model(&self.sweep_model, "reset_response", "Sweep config reset")
    }
}

#[cfg(test)]
mod tests {
    use super::DataModel;
    use crate::back_end::ipc_data::IpcData;
    use script_gen_manager::{
        device::Device,
        instr_metadata::base_metadata::BaseMetadata,
        model::{
            chan_data::sweep_channel::SweepChannel, sweep_data::sweep_model::SweepModel,
            system_info::Slot,
        },
    };
    use serde_json::json;

    #[test]
    fn model_aware_messages_reassign_and_release_only_the_selected_model() {
        let mut model = DataModel::new();
        let mut system = json!({
            "systems": [{
                "name": "System 1", "isActive": true, "localNode": "MP5103",
                "slots": [{"slotId": "slot[1]", "module": "MSMU60-2"}], "nodes": null
            }]
        });
        model.process_system_config(&system.to_string());
        system["systems"][0]["slots"][0]["module"] = json!("MSMU200-2");
        model.process_system_config(&system.to_string());
        let mut removal = model.clone();
        removal.sweep_model.sweep_config.add_channel("bias");
        removal.sweep_model.sweep_config.step_channels.clear();
        removal.add_remove_channel(IpcData {
            request_type: "add_remove_channel".into(),
            additional_info: "remove,step,localnode.slot[1].smu[1],MSMU60-2".into(),
            json_value: serde_json::to_string(&removal.sweep_model).unwrap(),
        });
        assert!(removal
            .sweep_model
            .sweep_config
            .device_list
            .iter()
            .any(|d| d.model == "MSMU200-2" && d.chan_num == 1 && d.in_use));
        assert!(!removal
            .sweep_model
            .sweep_config
            .device_list
            .iter()
            .any(|d| d.model == "MSMU60-2" && d.chan_num == 1));
        model.add_remove_channel(IpcData {
            request_type: "add_remove_channel".into(),
            additional_info:
                "update,step,localnode.slot[1].smu[1],localnode.slot[1].smu[1],MSMU60-2,MSMU200-2"
                    .into(),
            json_value: serde_json::to_string(&model.sweep_model).unwrap(),
        });
        let config = &model.sweep_model.sweep_config;
        let step = &config.step_channels[0]
            .start_stop_channel
            .common_chan_attributes;
        assert_eq!(step.device_model, "MSMU200-2");
        assert!(step.device.is_valid);
        assert!(
            !config.sweep_channels[0]
                .start_stop_channel
                .common_chan_attributes
                .device
                .is_valid
        );
    }

    #[test]
    fn rejected_operation_preserves_host_state_and_returns_error_status() {
        let mut model = DataModel::new();
        let original = serde_json::to_value(&model.sweep_model).unwrap();
        let response = model.add_remove_channel(IpcData {
            request_type: "add_remove_channel".into(),
            additional_info: "update,step,missing,missing,MSMU60-2,MSMU200-2".into(),
            json_value: serde_json::to_string(&model.sweep_model).unwrap(),
        });
        assert!(model.sweep_model.sweep_config.status_msg.is_some());
        let mut after = serde_json::to_value(&model.sweep_model).unwrap();
        after["sweep_config"]["status_msg"] = json!(null);
        assert_eq!(after, original);
        let envelope: IpcData = serde_json::from_str(&response).unwrap();
        let payload: serde_json::Value = serde_json::from_str(&envelope.json_value).unwrap();
        assert_eq!(
            payload["sweep_model"]["sweep_config"]["status_msg"]["status_type"],
            "Error"
        );
        for operation in [
            "update",
            "remove,step,address",
            "update,step,oldAddress,newAddress",
        ] {
            let mut submitted = model.sweep_model.clone();
            submitted
                .sweep_config
                .step_global_parameters
                .step_points
                .value = 123;
            let malformed = model.add_remove_channel(IpcData {
                request_type: "add_remove_channel".into(),
                additional_info: operation.into(),
                json_value: serde_json::to_string(&submitted).unwrap(),
            });
            let envelope: IpcData = serde_json::from_str(&malformed).unwrap();
            assert_eq!(envelope.request_type, "evaluated_response");
            let payload: serde_json::Value = serde_json::from_str(&envelope.json_value).unwrap();
            let mut returned = payload["sweep_model"].clone();
            assert_eq!(
                returned["sweep_config"]["status_msg"]["status_type"],
                "Error"
            );
            assert_eq!(
                returned["sweep_config"]["status_msg"]["message"],
                "Invalid channel operation. Existing configuration preserved."
            );
            returned["sweep_config"]["status_msg"] = json!(null);
            assert_eq!(returned, original);
            let mut actual = serde_json::to_value(&model.sweep_model).unwrap();
            actual["sweep_config"]["status_msg"] = json!(null);
            assert_eq!(actual, original);
        }
    }
    #[test]
    fn sweep_model_response_preserves_nested_ipc_shape() {
        let data_model = DataModel::new();

        let response = data_model.serialize_sweep_model(
            &data_model.sweep_model,
            "evaluated_response",
            "Processed sweep model",
        );

        let ipc_data: IpcData = serde_json::from_str(&response).unwrap();
        let inner: serde_json::Value = serde_json::from_str(&ipc_data.json_value).unwrap();
        assert_eq!(ipc_data.request_type, "evaluated_response");
        assert_eq!(ipc_data.additional_info, "Processed sweep model");
        assert!(inner.get("sweep_model").is_some());
        assert_eq!(inner.as_object().unwrap().len(), 1);
    }

    #[test]
    fn channel_id_update_response_clamps_200v_source_range_to_60v() {
        let old_device = Device::new(
            "localnode",
            "MP5103",
            &Slot {
                slot_id: "slot[1]".to_string(),
                module: "MSMU200-2".to_string(),
            },
            1,
        );
        let new_device = Device::new(
            "localnode",
            "MP5103",
            &Slot {
                slot_id: "slot[1]".to_string(),
                module: "MSMU60-2".to_string(),
            },
            2,
        );

        let mut sweep_model = SweepModel::new();
        let mut old_device = old_device;
        old_device.in_use = true;
        sweep_model.sweep_config.device_list = vec![old_device.clone(), new_device.clone()];
        let mut channel = SweepChannel::new("sweep1".to_string(), old_device.clone(), 10);
        channel
            .start_stop_channel
            .common_chan_attributes
            .source_function
            .value = BaseMetadata::FUNCTION_VOLTAGE.to_string();
        channel
            .start_stop_channel
            .common_chan_attributes
            .source_range
            .value = "200 V".to_string();
        sweep_model.sweep_config.sweep_channels.push(channel);

        let mut data_model = DataModel { sweep_model };
        let json_value = serde_json::to_string(&data_model.sweep_model).unwrap();
        let response = data_model.add_remove_channel(IpcData {
            request_type: "reallocation".to_string(),
            additional_info: format!(
                "update,sweep,{},{},MSMU200-2,MSMU60-2",
                old_device._id, new_device._id
            ),
            json_value,
        });

        let ipc_data: IpcData = serde_json::from_str(&response).unwrap();
        assert_eq!(ipc_data.request_type, "evaluated_response");
        let inner: serde_json::Value = serde_json::from_str(&ipc_data.json_value).unwrap();
        assert_eq!(
            inner["sweep_model"]["sweep_config"]["sweep_channels"][0]["start_stop_channel"]
                ["common_chan_attributes"]["source_function"]["value"],
            BaseMetadata::FUNCTION_VOLTAGE
        );
        assert_eq!(
            inner["sweep_model"]["sweep_config"]["sweep_channels"][0]["start_stop_channel"]
                ["common_chan_attributes"]["source_range"]["value"],
            "60 V"
        );
    }
}
