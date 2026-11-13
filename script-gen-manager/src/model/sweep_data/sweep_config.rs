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

                // Try localnode first
                let found_local = if system.local_node == "MP5103" {
                    let found = process_slots("localnode", &system.local_node, &system.slots);
                    if !found {
                        println!("All modules are empty in localnode. Checking nodes...");
                    }
                    found
                } else {
                    false
                };

                // If not found in localnode, check nodes
                if !found_local {
                    if let Some(nodes) = &system.nodes {
                        for node in nodes.iter().filter(|n| n.mainframe == "MP5103") {
                            let found = process_slots(&node.node_id, &node.mainframe, &node.slots);
                            if found {
                                break;
                            }
                            println!("All modules are empty in node {}. Skipping.", node.node_id);
                        }
                    } else {
                        println!("No nodes found in the system info.");
                    }
                }
            }
        } else if let Err(e) = res {
            println!("Error: {e:#?}");
        }

        is_device_found
    }

    pub fn update_devices_for_changed_slots(&mut self, system_info: &str) {
        self.status_msg = None;

        let res = serde_json::from_str::<Root>(system_info);
        let mut found_any_valid_slots = false;

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

                // Try localnode first
                if system.local_node == "MP5103" {
                    processed = process("localnode", &system.local_node, &system.slots);
                }

                // If not processed, try nodes
                if !processed {
                    if let Some(nodes) = &system.nodes {
                        if nodes.iter().any(|n| {
                            n.mainframe == "MP5103" && process(&n.node_id, &n.mainframe, &n.slots)
                        }) {
                            processed = true;
                        }
                    }
                }

                found_any_valid_slots |= processed;
            }
        } else if let Err(e) = res {
            println!("Error: {e:#?}");
        }

        if !found_any_valid_slots {
            for device in &mut self.device_list {
                device.is_valid = false;
            }
            println!("No valid slots found or system_info is empty. All devices invalidated.");
        }

        self.remove_unused_invalid_channels();

        if self.device_list.iter().any(|d| d.in_use && !d.is_valid) {
            self.status_msg = Some(StatusMsg::new(
            StatusType::Error,
            String::from("Some channels in use are invalid. Re-assign the invalid channels for the generated script to be functional"),
        ));
        }

        self.check_bias_only_configuration();
    }

    fn process_slots(&mut self, node_id: &str, mainframe: &str, slots: &[Slot]) {
        for slot in slots {
            // Find all indices of devices matching this slot_id
            let matching_indices: Vec<_> = self
                .device_list
                .iter()
                .enumerate()
                .filter_map(|(i, d)| {
                    if d.slot_id == slot.slot_id {
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
                if device.is_valid {
                    self.update_device_and_related_channels_node(idx, node_id);
                }
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

    /// Updates the node info for the device and all related channels if the `node_id` has changed.
    fn update_device_and_related_channels_node(&mut self, device_idx: usize, new_node_id: &str) {
        let device = &mut self.device_list[device_idx];
        if device.node_id == new_node_id {
            return;
        }
        let old_id = device._id.clone();
        device.update_node_info(new_node_id.to_string());
        let new_id = device._id.clone();
        let updated_device = device.clone();

        // Helper closure to update device_id and device
        let update_channel = |chan_device_id: &mut String, chan_device: &mut Device| {
            if *chan_device_id == old_id {
                chan_device_id.clone_from(&new_id);
                *chan_device = updated_device.clone();
            }
        };

        // Update bias channels
        for bias_channel in &mut self.bias_channels {
            update_channel(
                &mut bias_channel.common_chan_attributes.device_id,
                &mut bias_channel.common_chan_attributes.device,
            );
        }
        // Update step channels
        for step_channel in &mut self.step_channels {
            update_channel(
                &mut step_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device_id,
                &mut step_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device,
            );
        }
        // Update sweep channels
        for sweep_channel in &mut self.sweep_channels {
            update_channel(
                &mut sweep_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device_id,
                &mut sweep_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device,
            );
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
            ));
        }
    }

    /// Migrates the selected channel before updating device usage, leaving other channels intact.
    pub fn update_channel(&mut self, chan_type: &str, old_chan_id: &str, new_chan_id: &str) {
        if old_chan_id == new_chan_id {
            return;
        }

        let new_device_idx = self.device_list.iter().position(|d| d._id == new_chan_id);
        let old_device_idx = self.device_list.iter().position(|d| d._id == old_chan_id);

        let Some(new_idx) = new_device_idx else {
            return;
        };
        if !self.device_list[new_idx].is_valid {
            return;
        }
        let mut new_device = self.device_list[new_idx].clone();
        new_device.in_use = true;

        let updated = match chan_type {
            "bias" => self
                .bias_channels
                .iter_mut()
                .find(|chan| chan.common_chan_attributes.device_id == old_chan_id)
                .map(|chan| chan.migrate_to_device(new_device.clone()))
                .is_some(),
            "step" => self
                .step_channels
                .iter_mut()
                .find(|chan| {
                    chan.start_stop_channel.common_chan_attributes.device_id == old_chan_id
                })
                .map(|chan| {
                    chan.start_stop_channel
                        .migrate_to_device(new_device.clone());
                })
                .is_some(),
            "sweep" => self
                .sweep_channels
                .iter_mut()
                .find(|chan| {
                    chan.start_stop_channel.common_chan_attributes.device_id == old_chan_id
                })
                .map(|chan| {
                    chan.start_stop_channel
                        .migrate_to_device(new_device.clone());
                })
                .is_some(),
            _ => false,
        };

        if !updated {
            return;
        }

        // Release the old device only after the channel has migrated successfully.
        if let Some(old_idx) = old_device_idx {
            self.device_list[old_idx].in_use = false;
        }
        self.device_list[new_idx].in_use = true;
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
    use super::*;
    use crate::{
        instr_metadata::base_metadata::BaseMetadata,
        model::{
            chan_data::{
                bias_channel::BiasChannel, step_channel::StepChannel, sweep_channel::SweepChannel,
            },
            sweep_data::parameters::ParameterFloat,
            system_info::Slot,
        },
    };

    fn device(module: &str, channel: i32) -> Device {
        let slot = Slot {
            slot_id: "slot[1]".to_string(),
            module: module.to_string(),
        };
        Device::new("localnode", "MP5103", &slot, channel)
    }

    #[test]
    fn updating_same_module_channel_preserves_sweep_configuration() {
        let mut old_device = device("MSMU60-2", 1);
        old_device.in_use = true;
        let new_device = device("MSMU60-2", 2);
        let mut config = SweepConfig::new();
        config.device_list = vec![old_device.clone(), new_device.clone()];
        config.sweep_global_parameters.sweep_points.value = 3;
        config.sweep_global_parameters.list_sweep = true;

        let mut channel = SweepChannel::new("sweep1".to_string(), old_device.clone(), 3);
        let common = &mut channel.start_stop_channel.common_chan_attributes;
        common.source_range.value = "20 V".to_string();
        common.source_limiti.as_mut().unwrap().value = 0.05;
        channel.start_stop_channel.start.value = 2.0;
        channel.start_stop_channel.stop.value = 5.0;
        channel.start_stop_channel.style.value = BaseMetadata::STYLE_LOG.to_string();
        channel.start_stop_channel.list = vec![
            ParameterFloat::new("list_0", 1.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            ParameterFloat::new("list_1", 3.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            ParameterFloat::new("list_2", 4.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
        ];
        let uuid = channel
            .start_stop_channel
            .common_chan_attributes
            .uuid
            .clone();
        config.sweep_channels.push(channel);

        config.update_channel("sweep", &old_device._id, &new_device._id);
        assert!(
            config.sweep_channels[0]
                .start_stop_channel
                .common_chan_attributes
                .device
                .in_use
        );
        config.evaluate();

        let migrated = &config.sweep_channels[0].start_stop_channel;
        assert_eq!(migrated.common_chan_attributes.uuid, uuid);
        assert_eq!(migrated.common_chan_attributes.device_id, new_device._id);
        assert_eq!(migrated.common_chan_attributes.source_range.value, "20 V");
        assert!(
            (migrated
                .common_chan_attributes
                .source_limiti
                .as_ref()
                .unwrap()
                .value
                - 0.05)
                .abs()
                < 1e-9
        );
        assert_eq!(migrated.start.value, 2.0);
        assert_eq!(migrated.stop.value, 5.0);
        assert_eq!(migrated.style.value, BaseMetadata::STYLE_LOG);
        assert_eq!(
            migrated
                .list
                .iter()
                .map(|point| point.value)
                .collect::<Vec<_>>(),
            vec![1.0, 3.0, 4.0]
        );
        assert!(!config.device_list[0].in_use);
        assert!(config.device_list[1].in_use);
    }

    #[test]
    fn updating_to_a_different_module_clamps_bias_and_removes_unsupported_fields() {
        let mut old_device = device("MSMU60-2", 1);
        old_device.in_use = true;
        let new_device = device("MPSU50-2ST", 2);
        let mut config = SweepConfig::new();
        config.device_list = vec![old_device.clone(), new_device.clone()];

        let mut channel = BiasChannel::new("bias1".to_string(), old_device.clone());
        channel.bias.value = 200.0;
        config.bias_channels.push(channel);

        config.update_channel("bias", &old_device._id, &new_device._id);
        config.evaluate();

        let migrated = &config.bias_channels[0];
        assert_eq!(migrated.common_chan_attributes.device_id, new_device._id);
        assert!((migrated.bias.value - 50.1).abs() < 1e-9);
        assert!(migrated.common_chan_attributes.source_limitv.is_none());
        assert!(migrated.common_chan_attributes.sense_mode.is_none());
        assert!(!config.device_list[0].in_use);
        assert!(config.device_list[1].in_use);
    }

    #[test]
    fn updating_to_a_different_module_clamps_sweep_values_without_resetting_list_points() {
        let mut old_device = device("MSMU60-2", 1);
        old_device.in_use = true;
        let new_device = device("MPSU50-2ST", 2);
        let mut config = SweepConfig::new();
        config.device_list = vec![old_device.clone(), new_device.clone()];
        config.sweep_global_parameters.sweep_points.value = 3;
        config.sweep_global_parameters.list_sweep = true;

        let mut channel = SweepChannel::new("sweep1".to_string(), old_device.clone(), 3);
        channel.start_stop_channel.start.value = 1.0;
        channel.start_stop_channel.stop.value = 200.0;
        channel.start_stop_channel.list = vec![
            ParameterFloat::new("list_0", 10.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            ParameterFloat::new("list_1", 40.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            ParameterFloat::new("list_2", 200.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
        ];
        config.sweep_channels.push(channel);

        config.update_channel("sweep", &old_device._id, &new_device._id);
        config.evaluate();

        let migrated = &config.sweep_channels[0].start_stop_channel;
        assert_eq!(migrated.list.len(), 3);
        assert_eq!(
            migrated
                .list
                .iter()
                .map(|point| point.value)
                .collect::<Vec<_>>(),
            vec![10.0, 40.0, 50.1]
        );
        assert!((migrated.stop.value - 50.1).abs() < 1e-9);
    }

    #[test]
    fn updating_channel_clamps_unsupported_fixed_range_to_new_device_maximum() {
        let mut old_device = device("MSMU200-2", 1);
        old_device.in_use = true;
        let new_device = device("MSMU60-2", 2);
        let mut config = SweepConfig::new();
        config.device_list = vec![old_device.clone(), new_device.clone()];

        let mut channel = SweepChannel::new("sweep1".to_string(), old_device.clone(), 10);
        channel
            .start_stop_channel
            .common_chan_attributes
            .source_range
            .value = "200 V".to_string();
        config.sweep_channels.push(channel);

        // The UI sends JSON and the backend reconstructs device metadata before migration.
        config = serde_json::from_str(&serde_json::to_string(&config).unwrap()).unwrap();
        config.update_channel_devices();
        config.update_channel("sweep", &old_device._id, &new_device._id);
        config.evaluate();

        assert_eq!(
            config.sweep_channels[0]
                .start_stop_channel
                .common_chan_attributes
                .source_range
                .value,
            "60 V"
        );
    }

    #[test]
    fn updating_step_channel_preserves_its_configuration() {
        let mut old_device = device("MSMU60-2", 1);
        old_device.in_use = true;
        let new_device = device("MSMU60-2", 2);
        let mut config = SweepConfig::new();
        config.device_list = vec![old_device.clone(), new_device.clone()];
        config.step_global_parameters.step_points.value = 3;
        config.step_global_parameters.list_step = true;

        let mut channel = StepChannel::new("step1".to_string(), old_device.clone(), 3);
        channel.start_stop_channel.start.value = 2.0;
        channel.start_stop_channel.stop.value = 5.0;
        channel.start_stop_channel.list = vec![
            ParameterFloat::new("list_0", 1.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            ParameterFloat::new("list_1", 3.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            ParameterFloat::new("list_2", 4.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
        ];
        config.step_channels.push(channel);

        config.update_channel("step", &old_device._id, &new_device._id);
        config.evaluate();

        let migrated = &config.step_channels[0].start_stop_channel;
        assert_eq!(migrated.common_chan_attributes.device_id, new_device._id);
        assert_eq!(migrated.start.value, 2.0);
        assert_eq!(migrated.stop.value, 5.0);
        assert_eq!(
            migrated
                .list
                .iter()
                .map(|point| point.value)
                .collect::<Vec<_>>(),
            vec![1.0, 3.0, 4.0]
        );
    }

    #[test]
    fn unsupported_source_function_uses_target_defaults_without_dropping_list_entries() {
        let mut old_device = device("MSMU60-2", 1);
        old_device.in_use = true;
        let new_device = device("MPSU50-2ST", 2);
        let mut config = SweepConfig::new();
        config.device_list = vec![old_device.clone(), new_device.clone()];
        config.sweep_global_parameters.sweep_points.value = 3;
        config.sweep_global_parameters.list_sweep = true;

        let mut channel = SweepChannel::new("sweep1".to_string(), old_device.clone(), 3);
        let common = &mut channel.start_stop_channel.common_chan_attributes;
        common.source_function.value = BaseMetadata::FUNCTION_CURRENT.to_string();
        common.source_range.value = "100 mA".to_string();
        channel.start_stop_channel.start =
            ParameterFloat::new("start", 0.1, Some(BaseMetadata::UNIT_AMPERES.to_string()));
        channel.start_stop_channel.stop =
            ParameterFloat::new("stop", 0.5, Some(BaseMetadata::UNIT_AMPERES.to_string()));
        channel.start_stop_channel.list = vec![
            ParameterFloat::new("list_0", 0.1, Some(BaseMetadata::UNIT_AMPERES.to_string())),
            ParameterFloat::new("list_1", 0.3, Some(BaseMetadata::UNIT_AMPERES.to_string())),
            ParameterFloat::new("list_2", 0.5, Some(BaseMetadata::UNIT_AMPERES.to_string())),
        ];
        config.sweep_channels.push(channel);

        config.update_channel("sweep", &old_device._id, &new_device._id);
        config.evaluate();

        let migrated = &config.sweep_channels[0].start_stop_channel;
        assert_eq!(
            migrated.common_chan_attributes.source_function.value,
            BaseMetadata::FUNCTION_VOLTAGE
        );
        assert_eq!(migrated.start.value, 0.0);
        assert_eq!(migrated.stop.value, 1.0);
        assert_eq!(migrated.list.len(), 3);
        assert!(migrated.list.iter().all(|point| {
            point.value == 0.0 && point.unit.as_deref() == Some(BaseMetadata::UNIT_VOLTS)
        }));
    }

    #[test]
    fn selecting_the_current_channel_id_is_a_no_op() {
        let mut old_device = device("MSMU60-2", 1);
        old_device.in_use = true;
        let mut config = SweepConfig::new();
        config.device_list.push(old_device.clone());
        let mut channel = SweepChannel::new("sweep1".to_string(), old_device.clone(), 10);
        channel.start_stop_channel.stop.value = 42.0;
        config.sweep_channels.push(channel);

        config.update_channel("sweep", &old_device._id, &old_device._id);

        assert_eq!(config.sweep_channels[0].start_stop_channel.stop.value, 42.0);
        assert!(config.device_list[0].in_use);
    }
}
