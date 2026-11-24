use serde::{Deserialize, Serialize};

use crate::{
    device::Device,
    instr_metadata::{base_metadata::Metadata, enum_metadata::MetadataEnum},
    model::{
        chan_data::{
            bias_channel::BiasChannel, default_channel::CommonChanAttributes,
            step_channel::StepChannel, sweep_channel::SweepChannel,
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
        self.update_channel_devices();

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

    /// Restores the module model for channels saved before `device_model` existed.
    ///
    /// Legacy reassignment between modules at the same address never took effect, so the channel
    /// is still bound to the saved device at that address that is marked in use, even if a
    /// replacement module now shares the address. Must run before `evaluate()`.
    pub fn backfill_legacy_device_models(&mut self) {
        let device_list = &self.device_list;
        let channels = self
            .bias_channels
            .iter_mut()
            .map(|c| &mut c.common_chan_attributes)
            .chain(
                self.step_channels
                    .iter_mut()
                    .map(|c| &mut c.start_stop_channel.common_chan_attributes),
            )
            .chain(
                self.sweep_channels
                    .iter_mut()
                    .map(|c| &mut c.start_stop_channel.common_chan_attributes),
            );
        for channel in channels.filter(|c| c.device_model.is_empty()) {
            let mut in_use = device_list
                .iter()
                .filter(|device| device._id == channel.device_id && device.in_use);
            if let (Some(device), None) = (in_use.next(), in_use.next()) {
                channel.device_model.clone_from(&device.model);
            }
        }
    }

    /// Synchronizes assigned device copies using the address and module model.
    pub fn update_channel_devices(&mut self) {
        let channels = self
            .bias_channels
            .iter_mut()
            .map(|c| &mut c.common_chan_attributes)
            .chain(
                self.step_channels
                    .iter_mut()
                    .map(|c| &mut c.start_stop_channel.common_chan_attributes),
            )
            .chain(
                self.sweep_channels
                    .iter_mut()
                    .map(|c| &mut c.start_stop_channel.common_chan_attributes),
            );
        for channel in channels {
            let mut matches = self.device_list.iter().filter(|device| {
                device._id == channel.device_id && device.model == channel.device_model
            });
            if let (Some(device), None) = (matches.next(), matches.next()) {
                channel.device = device.clone();
            } else {
                channel.device.is_valid = false;
                let message = format!(
                    "Cannot uniquely resolve device {} (model '{}'). Re-assign the channel.",
                    channel.device_id, channel.device_model
                );
                println!("{message}");
                self.status_msg = Some(StatusMsg::new(StatusType::Error, message, Some(0)));
            }
        }
    }

    /// Releases only the selected address and module model.
    pub fn remove_channel_for_model(&mut self, chan_id: &str, model: &str) -> bool {
        let Some(index) = self.resolve_device_index(chan_id, model) else {
            return false;
        };
        self.device_list[index].in_use = false;

        self.check_bias_only_configuration();
        true
    }

    fn channel_operation_error(&mut self, message: String) {
        println!("{message}");
        self.status_msg = Some(StatusMsg::new(StatusType::Error, message, Some(0)));
    }

    fn resolve_device_index(&mut self, address: &str, model: &str) -> Option<usize> {
        if model.is_empty() {
            self.channel_operation_error(format!("Module model is required for device {address}."));
            return None;
        }
        let mut matches = self
            .device_list
            .iter()
            .enumerate()
            .filter(|(_, device)| device._id == address && device.model == model);
        if let (Some((index, _)), None) = (matches.next(), matches.next()) {
            return Some(index);
        }
        self.channel_operation_error(format!(
            "Cannot uniquely resolve device {address} (model '{model}'). Re-assign the channel."
        ));
        None
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

    /// Reassigns a channel using both device address and module model.
    pub fn update_channel_for_models(
        &mut self,
        chan_type: &str,
        old_chan_id: &str,
        new_chan_id: &str,
        old_model: &str,
        new_model: &str,
    ) -> bool {
        // Resolve by address and model so a replaced module at the same address is not mistaken
        // for the channel's original device.
        let Some(old_idx) = self.resolve_device_index(old_chan_id, old_model) else {
            return false;
        };
        let Some(new_idx) = self.resolve_device_index(new_chan_id, new_model) else {
            return false;
        };

        // Match the channel's current device by both address and model.
        let matches_channel = |channel: &CommonChanAttributes| {
            channel.device_id == old_chan_id && channel.device_model == old_model
        };

        // Count matches only within the requested channel type; reassignment must target one channel.
        let matching_channels = match chan_type {
            "bias" => self
                .bias_channels
                .iter()
                .filter(|channel| matches_channel(&channel.common_chan_attributes))
                .count(),
            "step" => self
                .step_channels
                .iter()
                .filter(|channel| {
                    matches_channel(&channel.start_stop_channel.common_chan_attributes)
                })
                .count(),
            "sweep" => self
                .sweep_channels
                .iter()
                .filter(|channel| {
                    matches_channel(&channel.start_stop_channel.common_chan_attributes)
                })
                .count(),
            _ => 0,
        };

        // Reject ambiguous channel matches and unavailable targets before changing either channel
        // data or device usage state.
        if matching_channels != 1
            || !self.device_list[new_idx].is_valid
            || (new_idx != old_idx && self.device_list[new_idx].in_use)
        {
            self.channel_operation_error(String::from(
                "Cannot reassign channel: assignment is missing or ambiguous, or the selected device is invalid or in use.",
            ));
            return false;
        }

        // Avoid rebuilding the channel when the selection has not actually changed.
        if old_idx == new_idx {
            return true;
        }

        let mut new_device = self.device_list[new_idx].clone();
        new_device.in_use = true;

        // Migration retains settings supported by the target device and adapts unsupported ones.
        let updated = match chan_type {
            "bias" => self
                .bias_channels
                .iter_mut()
                .find(|channel| matches_channel(&channel.common_chan_attributes))
                .map(|channel| channel.migrate_to_device(new_device))
                .is_some(),
            "step" => self
                .step_channels
                .iter_mut()
                .find(|channel| matches_channel(&channel.start_stop_channel.common_chan_attributes))
                .map(|channel| {
                    channel.start_stop_channel.migrate_to_device(new_device);
                })
                .is_some(),
            "sweep" => self
                .sweep_channels
                .iter_mut()
                .find(|channel| matches_channel(&channel.start_stop_channel.common_chan_attributes))
                .map(|channel| {
                    channel.start_stop_channel.migrate_to_device(new_device);
                })
                .is_some(),
            _ => false,
        };

        if !updated {
            self.channel_operation_error(String::from(
                "Cannot reassign channel: assignment is missing or ambiguous, or the selected device is invalid or in use.",
            ));
            return false;
        }

        // Release the old device only after the channel migration has succeeded.
        self.device_list[new_idx].in_use = true;
        self.device_list[old_idx].in_use = false;
        true
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
    use super::*;
    use crate::{
        instr_metadata::base_metadata::BaseMetadata,
        model::{
            chan_data::{step_channel::StepChannel, sweep_channel::SweepChannel},
            sweep_data::parameters::ParameterFloat,
            system_info::Slot,
        },
    };
    use serde_json::json;

    fn device(module: &str, channel: i32) -> Device {
        let slot = Slot {
            slot_id: "slot[1]".to_string(),
            module: module.to_string(),
        };
        Device::new("localnode", "MP5103", &slot, channel)
    }

    #[test]
    fn reassignment_migrates_sweep_settings_between_models_at_the_same_address() {
        let mut old_device = device("MSMU200-2", 1);
        old_device.in_use = true;
        let new_device = device("MSMU60-2", 1);
        let mut config = SweepConfig::new();
        config.device_list = vec![old_device.clone(), new_device.clone()];
        config.sweep_global_parameters.sweep_points.value = 3;
        config.sweep_global_parameters.list_sweep = true;

        let mut channel = SweepChannel::new("sweep1".to_string(), old_device.clone(), 3);
        let common = &mut channel.start_stop_channel.common_chan_attributes;
        common.source_range.value = "200 V".to_string();
        channel.start_stop_channel.start.value = 10.0;
        channel.start_stop_channel.stop.value = 180.0;
        channel.start_stop_channel.list = vec![
            ParameterFloat::new("list_0", 10.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            ParameterFloat::new("list_1", 100.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            ParameterFloat::new("list_2", 180.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
        ];
        let original_uuid = common.uuid.clone();
        config.sweep_channels.push(channel);

        assert!(config.update_channel_for_models(
            "sweep",
            &old_device._id,
            &new_device._id,
            &old_device.model,
            &new_device.model,
        ));

        let migrated = &config.sweep_channels[0].start_stop_channel;
        assert_eq!(
            migrated.common_chan_attributes.device_model,
            new_device.model
        );
        assert_eq!(migrated.common_chan_attributes.uuid, original_uuid);
        assert_eq!(migrated.common_chan_attributes.source_range.value, "60 V");
        assert!((migrated.stop.value - 60.6).abs() < 1e-9);
        assert_eq!(migrated.list.len(), 3);
        assert_eq!(
            migrated
                .list
                .iter()
                .map(|point| point.value)
                .collect::<Vec<_>>(),
            vec![10.0, 60.6, 60.6]
        );
        assert!(!config.device_list[0].in_use);
        assert!(config.device_list[1].in_use);
    }

    #[test]
    fn reassignment_preserves_step_configuration() {
        let mut old_device = device("MSMU60-2", 1);
        old_device.in_use = true;
        let new_device = device("MSMU60-2", 2);
        let mut config = SweepConfig::new();
        config.device_list = vec![old_device.clone(), new_device.clone()];

        let mut channel = StepChannel::new("step1".to_string(), old_device.clone(), 3);
        channel.start_stop_channel.start.value = 2.0;
        channel.start_stop_channel.stop.value = 5.0;
        channel.start_stop_channel.list = vec![
            ParameterFloat::new("list_0", 1.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            ParameterFloat::new("list_1", 3.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            ParameterFloat::new("list_2", 4.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
        ];
        config.step_channels.push(channel);

        assert!(config.update_channel_for_models(
            "step",
            &old_device._id,
            &new_device._id,
            &old_device.model,
            &new_device.model,
        ));

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
    fn reassignment_migrates_bias_settings_to_target_capabilities() {
        let mut old_device = device("MSMU60-2", 1);
        old_device.in_use = true;
        let new_device = device("MPSU50-2ST", 2);
        let mut config = SweepConfig::new();
        config.device_list = vec![old_device.clone(), new_device.clone()];

        let mut channel = BiasChannel::new("bias1".to_string(), old_device.clone());
        channel.bias.value = 200.0;
        config.bias_channels.push(channel);

        assert!(config.update_channel_for_models(
            "bias",
            &old_device._id,
            &new_device._id,
            &old_device.model,
            &new_device.model,
        ));

        let migrated = &config.bias_channels[0];
        assert_eq!(
            migrated.common_chan_attributes.device_model,
            new_device.model
        );
        assert!((migrated.bias.value - 50.1).abs() < 1e-9);
        assert!(migrated.common_chan_attributes.source_limitv.is_none());
        assert!(migrated.common_chan_attributes.sense_mode.is_none());
        assert!(!config.device_list[0].in_use);
        assert!(config.device_list[1].in_use);
    }

    #[test]
    fn rejected_reassignment_does_not_mutate_devices_or_channels() {
        for case in [
            "missing-target",
            "ambiguous-target",
            "invalid-target",
            "in-use-target",
            "missing-channel",
            "missing-model",
        ] {
            let mut config = SweepConfig::new();
            let mut old_device = device("MSMU60-2", 1);
            old_device.in_use = true;
            let new_device = device("MSMU60-2", 2);
            config.device_list = vec![old_device.clone(), new_device.clone()];
            config.step_channels.push(StepChannel::new(
                "step1".to_string(),
                old_device.clone(),
                10,
            ));

            match case {
                "missing-target" => {
                    config.device_list.pop();
                }
                "ambiguous-target" => config.device_list.push(new_device.clone()),
                "invalid-target" => config.device_list[1].is_valid = false,
                "in-use-target" => config.device_list[1].in_use = true,
                "missing-channel" => config.step_channels.clear(),
                "missing-model" => {
                    config.step_channels[0]
                        .start_stop_channel
                        .common_chan_attributes
                        .device_model
                        .clear();
                }
                _ => unreachable!(),
            }

            let before = json!([config.device_list, config.step_channels]);
            assert!(
                !config.update_channel_for_models(
                    "step",
                    &old_device._id,
                    &new_device._id,
                    &old_device.model,
                    &new_device.model,
                ),
                "{case}"
            );
            assert_eq!(json!([config.device_list, config.step_channels]), before);
            assert!(config.status_msg.is_some(), "{case}");
        }
    }

    #[test]
    fn operations_require_explicit_models() {
        let mut config = SweepConfig::new();
        let old_device = device("MSMU60-2", 1);
        let new_device = device("MSMU200-2", 1);
        config.device_list = vec![old_device.clone(), new_device.clone()];
        config.step_channels.push(StepChannel::new(
            "step1".to_string(),
            old_device.clone(),
            10,
        ));
        let before = json!([config.device_list, config.step_channels]);

        assert!(!config.remove_channel_for_model(&old_device._id, ""));
        assert!(!config.update_channel_for_models(
            "step",
            &old_device._id,
            &new_device._id,
            "",
            &new_device.model,
        ));
        assert!(!config.update_channel_for_models(
            "step",
            &old_device._id,
            &new_device._id,
            &old_device.model,
            "",
        ));
        assert_eq!(json!([config.device_list, config.step_channels]), before);
        assert!(config.status_msg.is_some());
    }

    #[test]
    fn refresh_keeps_assigned_channels_on_the_original_module_model() {
        let mut system = json!({
            "systems": [{
                "name": "System 1",
                "isActive": true,
                "localNode": "MP5103",
                "slots": [
                    { "slotId": "slot[1]", "module": "MSMU60-2" },
                    { "slotId": "slot[2]", "module": "MSMU60-2" }
                ],
                "nodes": null
            }]
        });
        let mut config = SweepConfig::new();
        assert!(config.create_device_list(&system.to_string()));
        config.auto_configure();

        for slot in system["systems"][0]["slots"].as_array_mut().unwrap() {
            slot["module"] = json!("MSMU200-2");
        }
        config.update_devices_for_changed_slots(&system.to_string());

        let channels = [
            &config.bias_channels[0].common_chan_attributes,
            &config.step_channels[0]
                .start_stop_channel
                .common_chan_attributes,
            &config.sweep_channels[0]
                .start_stop_channel
                .common_chan_attributes,
        ];
        for channel in channels {
            assert_eq!(channel.device_model, "MSMU60-2");
            assert_eq!(channel.device.model, "MSMU60-2");
            assert!(!channel.device.is_valid);
            assert!(channel.device.in_use);
        }
        assert!(config
            .device_list
            .iter()
            .any(|device| device.model == "MSMU200-2" && device.is_valid));
    }

    #[test]
    fn device_synchronization_rejects_missing_or_ambiguous_model_matches() {
        for case in ["missing", "ambiguous", "model-missing"] {
            let mut config = SweepConfig::new();
            config.process_slots(
                "localnode",
                "MP5103",
                &[Slot {
                    slot_id: "slot[1]".to_string(),
                    module: "MSMU60-2".to_string(),
                }],
            );
            config.add_channel("step");

            match case {
                "missing" => config.device_list.clear(),
                "ambiguous" => config.device_list.push(config.device_list[0].clone()),
                "model-missing" => config.step_channels[0]
                    .start_stop_channel
                    .common_chan_attributes
                    .device_model
                    .clear(),
                _ => unreachable!(),
            }

            config.update_channel_devices();
            assert!(
                !config.step_channels[0]
                    .start_stop_channel
                    .common_chan_attributes
                    .device
                    .is_valid,
                "{case}"
            );
            assert!(config.status_msg.is_some(), "{case}");
        }
    }

    #[test]
    fn selecting_the_current_device_is_a_no_op() {
        let mut current_device = device("MSMU60-2", 1);
        current_device.in_use = true;
        let mut config = SweepConfig::new();
        config.device_list.push(current_device.clone());
        let mut channel = SweepChannel::new("sweep1".to_string(), current_device.clone(), 10);
        channel.start_stop_channel.stop.value = 42.0;
        config.sweep_channels.push(channel);

        assert!(config.update_channel_for_models(
            "sweep",
            &current_device._id,
            &current_device._id,
            &current_device.model,
            &current_device.model,
        ));

        assert_eq!(config.sweep_channels[0].start_stop_channel.stop.value, 42.0);
        assert!(config.device_list[0].in_use);
    }
}
