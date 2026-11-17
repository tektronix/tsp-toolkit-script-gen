use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    device::Device,
    instr_metadata::{base_metadata::Metadata, enum_metadata::MetadataEnum},
    model::{
        chan_data::{
            bias_channel::BiasChannel, step_channel::StepChannel, sweep_channel::SweepChannel,
        },
        sweep_data::status_msg::{StatusMsg, StatusType},
        system_info::{Root, Slot},
    },
};

use super::{
    global_parameters::GlobalParameters,
    step_sweep_config::{StepGlobalParameters, SweepGlobalParameters},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SweepConfig {
    pub global_parameters: GlobalParameters,
    pub bias_channels: Vec<BiasChannel>,
    pub step_channels: Vec<StepChannel>,
    pub sweep_channels: Vec<SweepChannel>,
    pub step_global_parameters: StepGlobalParameters,
    pub sweep_global_parameters: SweepGlobalParameters,
    pub device_list: Vec<Device>,

    #[serde(skip_deserializing)]
    pub status_msg: Option<StatusMsg>,
    // #[serde(skip)]
    // line_frequency: i32,
    // #[serde(skip)]
    // base_metadata: BaseMetadata,
}

impl Default for SweepConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl SweepConfig {
    #[must_use]
    pub fn new() -> Self {
        Self {
            global_parameters: GlobalParameters::new(),
            bias_channels: Vec::new(),
            step_channels: Vec::new(),
            sweep_channels: Vec::new(),
            step_global_parameters: StepGlobalParameters::new(),
            sweep_global_parameters: SweepGlobalParameters::new(),
            device_list: Vec::new(),
            status_msg: None,
        }
    }

    /// Adds channels from localnode and all MP5103 mainframe nodes in active systems.
    pub fn create_device_list(&mut self, system_info: &str) -> bool {
        let mut is_device_found = false;
        let res = serde_json::from_str::<Root>(system_info);

        if let Ok(root) = res {
            for system in root.systems.into_iter().filter(|s| s.is_active) {
                // Helper closure to process slots
                let mut process_slots =
                    |node_id: &str, mainframe: &str, slots_opt: &Option<Vec<Slot>>| -> bool {
                        if let Some(slots) = slots_opt {
                            let mut found = false;
                            for slot in slots.iter().filter(|slot| slot.module != "Empty") {
                                for i in 1..=2 {
                                    self.device_list
                                        .push(Device::new(node_id, mainframe, slot, i));
                                    is_device_found = true;
                                    found = true;
                                }
                            }
                            found
                        } else {
                            false
                        }
                    };

                if system.local_node == "MP5103" {
                    let found = process_slots("localnode", &system.local_node, &system.slots);
                    if !found {
                        println!("All modules are empty in localnode. Checking nodes...");
                    }
                }

                if let Some(nodes) = &system.nodes {
                    for node in nodes.iter().filter(|n| n.mainframe == "MP5103") {
                        let found = process_slots(&node.node_id, &node.mainframe, &node.slots);
                        if !found {
                            println!("All modules are empty in node {}. Skipping.", node.node_id);
                        }
                    }
                } else {
                    println!("No nodes found in the system info.");
                }
            }
        } else if let Err(e) = res {
            println!("Error: {e:#?}");
        }

        is_device_found
    }

    /// Refreshes localnode and all MP5103 mainframe nodes with non-empty slots in active systems.
    /// Missing or mismatched devices are retained only while in use.
    pub fn update_devices_for_changed_slots(&mut self, system_info: &str) {
        self.status_msg = None;

        let res = serde_json::from_str::<Root>(system_info);
        let mut found_any_valid_slots = false;

        // Only devices confirmed by this snapshot become valid again.
        for device in &mut self.device_list {
            device.is_valid = false;
        }

        if let Ok(root) = res {
            for system in root.systems.into_iter().filter(|s| s.is_active) {
                let mut processed = false;

                // Helper to process slots for a given node
                let mut process =
                    |node_id: &str, mainframe: &str, slots: &Option<Vec<Slot>>| -> bool {
                        slots.as_ref().is_some_and(|slots| {
                            let valid = slots.iter().any(|s| s.module != "Empty");
                            if valid {
                                self.process_slots(node_id, mainframe, slots);
                            }
                            valid
                        })
                    };

                if system.local_node == "MP5103" {
                    processed = process("localnode", &system.local_node, &system.slots);
                }

                if let Some(nodes) = &system.nodes {
                    for node in nodes.iter().filter(|n| n.mainframe == "MP5103") {
                        processed |= process(&node.node_id, &node.mainframe, &node.slots);
                    }
                }

                found_any_valid_slots |= processed;
            }
        } else if let Err(e) = res {
            println!("Error: {e:#?}");
        }

        if !found_any_valid_slots {
            println!("No valid slots found or system_info is empty. All devices invalidated.");
        }

        self.remove_unused_invalid_channels();

        if self.device_list.iter().any(|d| d.in_use && !d.is_valid) {
            self.status_msg = Some(StatusMsg::new(
            StatusType::Error,
            String::from("Some channels in use are invalid. Re-assign the invalid channels for the generated script to be functional"),
            Some(0),
        ));
        }

        self.check_bias_only_configuration();
    }

    fn process_slots(&mut self, node_id: &str, mainframe: &str, slots: &[Slot]) {
        for slot in slots {
            // Match the slot within its node, not across mainframes.
            let matching_indices: Vec<_> = self
                .device_list
                .iter()
                .enumerate()
                .filter_map(|(i, d)| {
                    if d.node_id == node_id && d.slot_id == slot.slot_id {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();

            // 1. If slot is empty, invalidate all matching devices and continue
            if slot.module == "Empty" {
                for &idx in &matching_indices {
                    self.device_list[idx].is_valid = false;
                }
                continue;
            }

            // 2. Validate/invalidate devices based on model
            for &idx in &matching_indices {
                let device = &mut self.device_list[idx];
                device.is_valid = device.model == slot.module;
            }

            // 3. Ensure at most one valid device for each channel (1 and 2)
            let mut valid_channels: Vec<i32> = matching_indices
                .iter()
                .filter_map(|&idx| {
                    let device = &self.device_list[idx];
                    if device.is_valid {
                        Some(device.chan_num)
                    } else {
                        None
                    }
                })
                .collect();

            for channel in 1..=2 {
                if !valid_channels.contains(&channel) {
                    let new_device = Device::new(node_id, mainframe, slot, channel);
                    self.device_list.push(new_device);
                    valid_channels.push(channel);
                }
            }
        }
    }

    pub fn auto_configure(&mut self) {
        //initially start with multi-sweep

        // Add default step
        let step_device = self
            .device_list
            .iter_mut()
            .find(|d| d.is_valid && !d.in_use)
            .map(|d| {
                d.in_use = true;
                d.clone()
            });

        if let Some(device) = step_device {
            let step_points = self.step_global_parameters.step_points.value;
            self.add_step(StepChannel::new(String::from("step1"), device, step_points));
        }

        // Add default sweep
        let sweep_device = self
            .device_list
            .iter_mut()
            .find(|d| d.is_valid && !d.in_use)
            .map(|d| {
                d.in_use = true;
                d.clone()
            });

        if let Some(device) = sweep_device {
            let sweep_points = self.sweep_global_parameters.sweep_points.value;
            self.add_sweep(SweepChannel::new(
                String::from("sweep1"),
                device,
                sweep_points,
            ));
        }

        // Add default bias
        let bias_device = self
            .device_list
            .iter_mut()
            .find(|d| d.is_valid && !d.in_use)
            .map(|d| {
                d.in_use = true;
                d.clone()
            });

        if let Some(device) = bias_device {
            self.add_bias(BiasChannel::new(String::from("bias1"), device));
        }
    }

    pub fn add_bias(&mut self, bias_chan: BiasChannel) {
        self.bias_channels.push(bias_chan);
    }

    pub fn add_step(&mut self, step_chan: StepChannel) {
        self.step_channels.push(step_chan);
    }

    pub fn add_sweep(&mut self, sweep_chan: SweepChannel) {
        self.sweep_channels.push(sweep_chan);
    }

    pub fn evaluate(&mut self) {
        self.update_channel_devices();
        self.global_parameters.evaluate();

        // Only one step channel can ever exist, so once it is removed the step count is one.
        let minimum_step_points = if self.step_channels.is_empty() {
            self.step_global_parameters.step_points.value = 1;
            1
        } else {
            2
        };

        self.step_global_parameters
            .step_points
            .limit(minimum_step_points, 60000);
        self.sweep_global_parameters.sweep_points.limit(2, 60000);
        self.global_parameters
            .sweep_timing_config
            .measure_count
            .limit(1, 60000);

        // Validating Step to Sweep Delay
        if !self.step_channels.is_empty() {
            let device_metadata = self.step_channels[0]
                .start_stop_channel
                .common_chan_attributes
                .device
                .get_metadata();
            if let Some((min, max)) =
                Self::get_range_limits(&device_metadata, "source.step_to_sweep_delay")
            {
                self.step_global_parameters
                    .step_to_sweep_delay
                    .limit(min, max);
            }
        }

        for bias_channel in &mut self.bias_channels {
            bias_channel.evaluate();
        }

        for step_channel in &mut self.step_channels {
            step_channel.start_stop_channel.evaluate(
                self.step_global_parameters.step_points.value as usize,
                self.step_global_parameters.list_step,
            );
        }
        for sweep_channel in &mut self.sweep_channels {
            sweep_channel.start_stop_channel.evaluate(
                self.sweep_global_parameters.sweep_points.value as usize,
                self.sweep_global_parameters.list_sweep,
            );
        }
    }

    pub fn update_channel_devices(&mut self) {
        let device_map: HashMap<String, Device> = self
            .device_list
            .iter()
            .map(|device| (device._id.clone(), device.clone()))
            .collect();

        // Update bias channels
        for bias_channel in &mut self.bias_channels {
            if let Some(device) = device_map.get(&bias_channel.common_chan_attributes.device_id) {
                bias_channel.common_chan_attributes.device = device.clone();
            }
        }

        // Update step channels
        for step_channel in &mut self.step_channels {
            if let Some(device) = device_map.get(
                &step_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device_id,
            ) {
                step_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device = device.clone();
            }
        }

        // Update sweep channels
        for sweep_channel in &mut self.sweep_channels {
            if let Some(device) = device_map.get(
                &sweep_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device_id,
            ) {
                sweep_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device = device.clone();
            }
        }
    }

    pub fn remove_channel(&mut self, chan_id: &str) {
        self.device_list.iter_mut().for_each(|device| {
            if device._id == chan_id {
                device.in_use = false;
            }
        });

        self.check_bias_only_configuration();
    }

    pub fn add_channel(&mut self, chan_type: &str) {
        // Find the first device that is valid and not in use
        if let Some(device_index) = self
            .device_list
            .iter()
            .position(|device| device.is_valid && !device.in_use)
        {
            // Temporarily take the device out of the list
            let mut device = self.device_list[device_index].clone();
            device.in_use = true; // Mark the device as in use

            // Reinsert the updated device back into the list
            self.device_list[device_index] = device.clone();

            if chan_type == "bias" {
                self.add_bias(BiasChannel::new(
                    format!("bias{}", self.bias_channels.len() + 1),
                    device,
                ));
            } else if chan_type == "step" {
                if self.step_channels.is_empty() {
                    self.step_global_parameters.step_points.value = 10;
                }
                self.add_step(StepChannel::new(
                    format!("step{}", self.step_channels.len() + 1),
                    device,
                    self.step_global_parameters.step_points.value,
                ));
            } else if chan_type == "sweep" {
                self.add_sweep(SweepChannel::new(
                    format!("sweep{}", self.sweep_channels.len() + 1),
                    device,
                    self.sweep_global_parameters.sweep_points.value,
                ));
            }
        } else {
            //add a status message if no valid or free device is found
            self.status_msg = Some(StatusMsg::new(
                StatusType::Warning,
                String::from("No valid or free device found to add a new channel."),
                None,
            ));
        }
    }

    pub fn update_channel(&mut self, chan_type: &str, old_chan_id: &str, new_chan_id: &str) {
        let new_device_idx = self.device_list.iter().position(|d| d._id == new_chan_id);
        let old_device_idx = self.device_list.iter().position(|d| d._id == old_chan_id);

        match new_device_idx {
            Some(new_idx) if self.device_list[new_idx].is_valid => {
                // Set old device as not in use
                if let Some(old_idx) = old_device_idx {
                    self.device_list[old_idx].in_use = false;
                }
                // Set new device as in use
                self.device_list[new_idx].in_use = true;
                let new_device = self.device_list[new_idx].clone();

                if chan_type == "bias" {
                    if let Some(bias_channel) = self
                        .bias_channels
                        .iter_mut()
                        .find(|chan| chan.common_chan_attributes.device_id == old_chan_id)
                    {
                        let mut new_bias_channel = BiasChannel::new(
                            bias_channel.common_chan_attributes.chan_name.clone(),
                            new_device,
                        );
                        new_bias_channel
                            .common_chan_attributes
                            .uuid
                            .clone_from(&bias_channel.common_chan_attributes.uuid);
                        *bias_channel = new_bias_channel;
                    }
                } else if chan_type == "step" {
                    if let Some(step_channel) = self.step_channels.iter_mut().find(|chan| {
                        chan.start_stop_channel.common_chan_attributes.device_id == old_chan_id
                    }) {
                        let mut new_step_channel = StepChannel::new(
                            step_channel
                                .start_stop_channel
                                .common_chan_attributes
                                .chan_name
                                .clone(),
                            new_device,
                            self.step_global_parameters.step_points.value,
                        );
                        new_step_channel
                            .start_stop_channel
                            .common_chan_attributes
                            .uuid
                            .clone_from(
                                &step_channel.start_stop_channel.common_chan_attributes.uuid,
                            );
                        *step_channel = new_step_channel;
                    }
                } else if chan_type == "sweep" {
                    if let Some(sweep_channel) = self.sweep_channels.iter_mut().find(|chan| {
                        chan.start_stop_channel.common_chan_attributes.device_id == old_chan_id
                    }) {
                        let mut new_sweep_channel = SweepChannel::new(
                            sweep_channel
                                .start_stop_channel
                                .common_chan_attributes
                                .chan_name
                                .clone(),
                            new_device,
                            self.sweep_global_parameters.sweep_points.value,
                        );
                        new_sweep_channel
                            .start_stop_channel
                            .common_chan_attributes
                            .uuid
                            .clone_from(
                                &sweep_channel.start_stop_channel.common_chan_attributes.uuid,
                            );
                        *sweep_channel = new_sweep_channel;
                    }
                }
            }
            _ => {
                // If the new device is not valid or not found, we can log an error or update the status message
            }
        }
    }

    pub fn remove_unused_invalid_channels(&mut self) {
        self.device_list
            .retain(|device| device.is_valid || device.in_use);
    }

    /// Checks if only bias channels are configured and sets appropriate warning message
    fn check_bias_only_configuration(&mut self) {
        if !self.bias_channels.is_empty()
            && self.step_channels.is_empty()
            && self.sweep_channels.is_empty()
        {
            self.status_msg = Some(StatusMsg::new(
                StatusType::Warning,
                String::from("Only bias channels are configured. Please add a step or sweep channel to generate a functional script."),
                None,
            ));
        }
    }

    fn get_range_limits(metadata: &MetadataEnum, key: &str) -> Option<(f64, f64)> {
        match metadata {
            MetadataEnum::Base(base_metadata) => base_metadata.get_range(key),
            MetadataEnum::Msmu60(msmu60_metadata) => msmu60_metadata.get_range(key),
            MetadataEnum::Msmu200(msmu200_metadata) => msmu200_metadata.get_range(key),
            MetadataEnum::Mpsu50(mpsu50_metadata) => mpsu50_metadata.get_range(key),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::SweepConfig;
    use crate::model::system_info::Slot;
    use serde_json::json;

    #[test]
    fn refresh_retains_missing_devices_only_when_in_use() {
        let original = json!({
            "systems": [{
                "name": "System 1", "isActive": true, "localNode": "MP5103",
                "slots": [{ "slotId": "slot[1]", "module": "MSMU60-2" }],
                "nodes": [{
                    "nodeId": "node[2]", "mainframe": "MP5103",
                    "slots": [{ "slotId": "slot[1]", "module": "MSMU60-2" }]
                }]
            }]
        });
        for target in ["localnode", "node[2]"] {
            for change in [
                "empty",
                "no-slots",
                "null-slots",
                "mainframe",
                "inactive",
                "no-systems",
                "removed",
            ] {
                if target == "localnode" && change == "removed" {
                    continue;
                }
                let context = format!("{target}: {change}");
                let mut config = SweepConfig::new();
                assert!(config.create_device_list(&original.to_string()));
                config.add_channel("step");
                let retained_id = format!("{target}.slot[1].smu[1]");
                if target != "localnode" {
                    config.update_channel("step", "localnode.slot[1].smu[1]", &retained_id);
                }
                let original_assignment = serde_json::to_value(&config.step_channels).unwrap();
                let mut updated = original.clone();
                let system = &mut updated["systems"][0];
                let slots = if target == "localnode" {
                    &mut system["slots"]
                } else {
                    &mut system["nodes"][0]["slots"]
                };
                match change {
                    "empty" => slots[0]["module"] = json!("Empty"),
                    "no-slots" => *slots = json!([]),
                    "null-slots" => *slots = json!(null),
                    "mainframe" if target == "localnode" => system["localNode"] = json!("2450"),
                    "mainframe" => system["nodes"][0]["mainframe"] = json!("2450"),
                    "inactive" => system["isActive"] = json!(false),
                    "no-systems" => updated["systems"] = json!([]),
                    "removed" => system["nodes"] = json!([]),
                    _ => unreachable!(),
                }
                for _ in 0..2 {
                    config.update_devices_for_changed_slots(&updated.to_string());
                    let retained = config
                        .device_list
                        .iter()
                        .find(|d| d._id == retained_id)
                        .unwrap();
                    assert!(retained.in_use && !retained.is_valid, "{context}");
                    let expected_valid = if matches!(change, "inactive" | "no-systems") {
                        0
                    } else {
                        2
                    };
                    assert_eq!(config.device_list.len(), expected_valid + 1, "{context}");
                    assert_eq!(
                        config.device_list.iter().filter(|d| d.is_valid).count(),
                        expected_valid,
                        "{context}"
                    );
                    assert!(
                        config.device_list.iter().all(|d| d.is_valid || d.in_use),
                        "{context}"
                    );
                    assert_eq!(
                        serde_json::to_value(&config.step_channels).unwrap(),
                        original_assignment,
                        "{context}"
                    );
                    assert!(config.status_msg.is_some(), "{context}");
                }
                config.update_devices_for_changed_slots(&original.to_string());
                assert_eq!(config.device_list.len(), 4, "{context}");
                assert!(config.device_list.iter().all(|d| d.is_valid), "{context}");
            }
        }
    }

    #[test]
    fn refresh_processes_all_qualifying_mainframes_without_duplicates() {
        for local_model in ["MP5103", "2450"] {
            let mut system_info = json!({
                "systems": [{
                    "name": "System 1", "isActive": true,
                    "localNode": local_model,
                    "slots": [{ "slotId": "slot[1]", "module": "MSMU60-2" }],
                    "nodes": [{
                        "nodeId": "node[2]", "mainframe": "MP5103",
                        "slots": [{ "slotId": "slot[1]", "module": "MSMU60-2" }]
                    }]
                }]
            });
            let mut config = SweepConfig::new();
            assert!(config.create_device_list(&system_info.to_string()));
            config.auto_configure();
            let original_channels = json!([
                config.bias_channels,
                config.step_channels,
                config.sweep_channels
            ]);
            system_info["systems"][0]["nodes"] = json!([
                { "nodeId": "node[1]", "mainframe": "MP5103",
                  "slots": [{ "slotId": "slot[1]", "module": "Empty" }] },
                { "nodeId": "node[2]", "mainframe": "MP5103",
                  "slots": [{ "slotId": "slot[1]", "module": "MSMU60-2" }] },
                { "nodeId": "node[3]", "mainframe": "2450",
                  "slots": [{ "slotId": "slot[1]", "module": "MSMU60-2" }] },
                { "nodeId": "node[4]", "mainframe": "MP5103",
                  "slots": [{ "slotId": "slot[1]", "module": "MSMU200-2" }] }
            ]);
            let mut expected = Vec::new();
            if local_model == "MP5103" {
                expected.extend(["localnode.slot[1].smu[1]", "localnode.slot[1].smu[2]"]);
            }
            expected.extend([
                "node[2].slot[1].smu[1]",
                "node[2].slot[1].smu[2]",
                "node[4].slot[1].smu[1]",
                "node[4].slot[1].smu[2]",
            ]);

            for _ in 0..2 {
                config.update_devices_for_changed_slots(&system_info.to_string());
                let actual: Vec<_> = config.device_list.iter().map(|d| d._id.as_str()).collect();
                assert_eq!(actual, expected);
                assert!(config.device_list.iter().all(|d| d.is_valid));
                assert_eq!(
                    json!([
                        config.bias_channels,
                        config.step_channels,
                        config.sweep_channels
                    ]),
                    original_channels
                );
            }
        }
    }

    #[test]
    fn process_slots_preserves_other_nodes_and_their_assignments() {
        for module in ["MSMU60-2", "MSMU200-2", "Empty"] {
            let mut config = SweepConfig::new();
            let local_slots = [
                Slot {
                    slot_id: "slot[1]".into(),
                    module: "MSMU60-2".into(),
                },
                Slot {
                    slot_id: "slot[2]".into(),
                    module: "MSMU60-2".into(),
                },
            ];
            config.process_slots("localnode", "MP5103", &local_slots);
            config.auto_configure();
            let original_devices = serde_json::to_value(&config.device_list).unwrap();
            let original_channels = json!([
                config.bias_channels,
                config.step_channels,
                config.sweep_channels
            ]);
            let remote_slots = [Slot {
                slot_id: "slot[1]".into(),
                module: module.into(),
            }];

            for _ in 0..2 {
                config.process_slots("node[2]", "MP5103", &remote_slots);
                assert_eq!(
                    serde_json::to_value(&config.device_list[..4]).unwrap(),
                    original_devices
                );
                assert_eq!(
                    json!([
                        config.bias_channels,
                        config.step_channels,
                        config.sweep_channels
                    ]),
                    original_channels
                );
                let expected_remote_count = if module == "Empty" { 0 } else { 2 };
                assert_eq!(config.device_list.len(), 4 + expected_remote_count);
                for (i, device) in config.device_list[4..].iter().enumerate() {
                    assert_eq!(device.node_id, "node[2]");
                    assert_eq!(device._id, format!("node[2].slot[1].smu[{}]", i + 1));
                    assert_eq!(device.model, module);
                    assert!(device.is_valid);
                }
            }
        }
    }

    #[test]
    fn create_device_list_includes_all_qualifying_mainframes() {
        for (local_model, local_module) in [
            ("MP5103", "MSMU60-2"),
            ("MP5103", "Empty"),
            ("2450", "MSMU60-2"),
        ] {
            let system_info = json!({
                "systems": [{
                    "name": "System 1",
                    "isActive": true,
                    "localNode": local_model,
                    "slots": [{ "slotId": "slot[1]", "module": local_module }],
                    "nodes": [
                        { "nodeId": "node[1]", "mainframe": "MP5103",
                          "slots": [{ "slotId": "slot[1]", "module": "Empty" }] },
                        { "nodeId": "node[2]", "mainframe": "MP5103",
                          "slots": [{ "slotId": "slot[1]", "module": "MSMU60-2" }] },
                        { "nodeId": "node[3]", "mainframe": "2450",
                          "slots": [{ "slotId": "slot[1]", "module": "MSMU60-2" }] },
                        { "nodeId": "node[4]", "mainframe": "MP5103",
                          "slots": [{ "slotId": "slot[1]", "module": "MSMU60-2" }] }
                    ]
                }, {
                    "name": "Inactive system",
                    "isActive": false,
                    "localNode": "MP5103",
                    "slots": [{ "slotId": "slot[2]", "module": "MSMU60-2" }],
                    "nodes": null
                }]
            });
            let mut config = SweepConfig::new();
            assert!(config.create_device_list(&system_info.to_string()));
            let mut expected = Vec::new();
            if local_model == "MP5103" && local_module != "Empty" {
                expected.extend(["localnode.slot[1].smu[1]", "localnode.slot[1].smu[2]"]);
            }
            expected.extend([
                "node[2].slot[1].smu[1]",
                "node[2].slot[1].smu[2]",
                "node[4].slot[1].smu[1]",
                "node[4].slot[1].smu[2]",
            ]);
            let actual: Vec<_> = config.device_list.iter().map(|d| d._id.as_str()).collect();
            assert_eq!(actual, expected);
        }
    }
}
