use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    device::Device,
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
    pub fn new() -> Self {
        SweepConfig {
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
                                    self.device_list.push(Device::new(
                                        node_id.to_string(),
                                        mainframe.to_string(),
                                        slot,
                                        i,
                                    ));
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
                            } else {
                                println!(
                                    "All modules are empty in node {}. Skipping.",
                                    node.node_id
                                );
                            }
                        }
                    } else {
                        println!("No nodes found in the system info.");
                    }
                }
            }
        } else if let Err(e) = res {
            println!("Error: {:#?}", e);
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
                        if let Some(slots) = slots {
                            let valid = slots.iter().any(|s| s.module != "Empty");
                            if valid {
                                self.process_slots(node_id, mainframe, slots);
                            }
                            valid
                        } else {
                            false
                        }
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
            println!("Error: {:#?}", e);
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
            String::from("Some channels in use are invalid. Re-assign the invalid channels for the generated script to be functional."),
        ));
        }
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
                    let new_device =
                        Device::new(node_id.to_string(), mainframe.to_string(), slot, channel);
                    self.device_list.push(new_device);
                    valid_channels.push(channel);
                }
            }
        }
    }

    /// Updates the node info for the device and all related channels if the node_id has changed.
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
                *chan_device_id = new_id.clone();
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
        self.step_global_parameters.step_points.limit(1, 60000);
        self.sweep_global_parameters.sweep_points.limit(1, 60000);
        self.global_parameters
            .sweep_timing_config
            .measure_count
            .limit(1, 60000);
        for bias_channel in &mut self.bias_channels {
            bias_channel.evaluate();
        }
        for step_channel in &mut self.step_channels {
            step_channel
                .start_stop_channel
                .evaluate(self.step_global_parameters.step_points.value as usize);
        }
        for sweep_channel in &mut self.sweep_channels {
            sweep_channel
                .start_stop_channel
                .evaluate(self.sweep_global_parameters.sweep_points.value as usize);
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

    pub fn remove_channel(&mut self, chan_id: String) {
        self.device_list.iter_mut().for_each(|device| {
            if device._id == chan_id {
                device.in_use = false;
            }
        });
    }

    pub fn add_channel(&mut self, chan_type: String) {
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
                    device.clone(),
                ));
            } else if chan_type == "step" {
                self.add_step(StepChannel::new(
                    format!("step{}", self.step_channels.len() + 1),
                    device.clone(),
                    self.step_global_parameters.step_points.value,
                ));
            } else if chan_type == "sweep" {
                self.add_sweep(SweepChannel::new(
                    format!("sweep{}", self.sweep_channels.len() + 1),
                    device.clone(),
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

    pub fn update_channel(&mut self, chan_type: String, old_chan_id: String, new_chan_id: String) {
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
                            new_device.clone(),
                        );
                        new_bias_channel.common_chan_attributes.uuid =
                            bias_channel.common_chan_attributes.uuid.clone();
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
                            new_device.clone(),
                            self.step_global_parameters.step_points.value,
                        );
                        new_step_channel
                            .start_stop_channel
                            .common_chan_attributes
                            .uuid = step_channel
                            .start_stop_channel
                            .common_chan_attributes
                            .uuid
                            .clone();
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
                            new_device.clone(),
                            self.sweep_global_parameters.sweep_points.value,
                        );
                        new_sweep_channel
                            .start_stop_channel
                            .common_chan_attributes
                            .uuid = sweep_channel
                            .start_stop_channel
                            .common_chan_attributes
                            .uuid
                            .clone();
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

    pub fn reset(&mut self) {
        *self = SweepConfig::new();
    }
}
