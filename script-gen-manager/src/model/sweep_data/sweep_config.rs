use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    device::Device,
    model::chan_data::{
        bias_channel::BiasChannel, step_channel::StepChannel, sweep_channel::SweepChannel,
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
    #[serde(skip)]
    total_chan_count: usize,
    pub step_global_parameters: StepGlobalParameters,
    pub sweep_global_parameters: SweepGlobalParameters,
    pub device_list: Vec<Device>,
    // #[serde(skip)]
    // line_frequency: i32,
    // #[serde(skip)]
    // base_metadata: BaseMetadata,
}

impl SweepConfig {
    pub fn new() -> Self {
        SweepConfig {
            global_parameters: GlobalParameters::new(),
            bias_channels: Vec::new(),
            step_channels: Vec::new(),
            sweep_channels: Vec::new(),
            total_chan_count: 0,
            step_global_parameters: StepGlobalParameters::new(),
            sweep_global_parameters: SweepGlobalParameters::new(),
            device_list: Vec::new(),
        }
    }

    pub fn auto_configure(&mut self) {
        //initially start with multi-sweep

        //add default step
        if self.can_add_channel() {
            self.device_list[self.total_chan_count].in_use = true;
            self.add_step(StepChannel::new(
                String::from("step1"),
                self.device_list[self.total_chan_count].clone(),
            ));
        }

        //add default sweep
        if self.can_add_channel() {
            self.device_list[self.total_chan_count].in_use = true;
            self.add_sweep(SweepChannel::new(
                String::from("sweep1"),
                self.device_list[self.total_chan_count].clone(),
            ));
        }

        //add default bias
        if self.can_add_channel() {
            self.device_list[self.total_chan_count].in_use = true;
            self.add_bias(BiasChannel::new(
                String::from("bias1"),
                self.device_list[self.total_chan_count].clone(),
            ));
        }
    }

    pub fn add_bias(&mut self, bias_chan: BiasChannel) {
        self.bias_channels.push(bias_chan);
        self.total_chan_count += 1;
    }

    pub fn add_step(&mut self, step_chan: StepChannel) {
        self.step_channels.push(step_chan);
        self.total_chan_count += 1;
    }

    pub fn add_sweep(&mut self, sweep_chan: SweepChannel) {
        self.sweep_channels.push(sweep_chan);
        self.total_chan_count += 1;
    }

    fn can_add_channel(&self) -> bool {
        self.device_list.len() > self.total_chan_count
    }

    pub fn evaluate(&mut self) {
        self.update_channel_devices();
        self.global_parameters.evaluate();
        for bias_channel in &mut self.bias_channels {
            bias_channel.evaluate();
        }
        for step_channel in &mut self.step_channels {
            step_channel.start_stop_channel.evaluate();
        }
        for sweep_channel in &mut self.sweep_channels {
            sweep_channel.start_stop_channel.evaluate();
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
                return;
            }
        });
    }

    pub fn add_channel(&mut self, chan_type: String) {
        if self.can_add_channel() {
            // Find the first free device (in_use = false)
            if let Some(device_index) = self.device_list.iter().position(|device| !device.in_use) {
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
                    ));
                } else if chan_type == "sweep" {
                    self.add_sweep(SweepChannel::new(
                        format!("sweep{}", self.sweep_channels.len() + 1),
                        device.clone(),
                    ));
                }
            } else {
                //TODO: Show error message in the UI
                println!("No free device available to add a new channel.");
            }
        } else {
            //TODO: Show error message in the UI
            println!("Cannot add channel: Maximum channel count reached.");
        }
    }

    pub fn update_channel(&mut self, chan_type: String, old_chan_id: String, new_chan_id: String) {
        // Find the first device with a matching old_chan_id and set in_use to false
        if let Some(device) = self
            .device_list
            .iter_mut()
            .find(|device| device._id == old_chan_id)
        {
            device.in_use = false;
        }

        // Find the first device with a matching new_chan_id and set in_use to true
        if let Some(device) = self
            .device_list
            .iter_mut()
            .find(|device| device._id == new_chan_id)
        {
            device.in_use = true;

            if chan_type == "bias" {
                if let Some(bias_channel) = self
                    .bias_channels
                    .iter_mut()
                    .find(|chan| chan.common_chan_attributes.device_id == old_chan_id)
                {
                    // Create a new BiasChannel using the device and chan_name
                    let mut new_bias_channel = BiasChannel::new(
                        bias_channel.common_chan_attributes.chan_name.clone(),
                        device.clone(),
                    );
                    //cloning old uuid since we are not actually creating a new channel
                    new_bias_channel.common_chan_attributes.uuid =
                        bias_channel.common_chan_attributes.uuid.clone();

                    // Update the existing bias_channel in the list
                    *bias_channel = new_bias_channel;
                }
            } else if chan_type == "step" {
                if let Some(step_channel) = self.step_channels.iter_mut().find(|chan| {
                    chan.start_stop_channel.common_chan_attributes.device_id == old_chan_id
                }) {
                    // Create a new StepChannel using the device and chan_name
                    let mut new_step_channel = StepChannel::new(
                        step_channel
                            .start_stop_channel
                            .common_chan_attributes
                            .chan_name
                            .clone(),
                        device.clone(),
                    );
                    new_step_channel
                        .start_stop_channel
                        .common_chan_attributes
                        .uuid = step_channel
                        .start_stop_channel
                        .common_chan_attributes
                        .uuid
                        .clone();

                    // Update the existing step_channel in the list
                    *step_channel = new_step_channel;
                }
            } else if chan_type == "sweep" {
                if let Some(sweep_channel) = self.sweep_channels.iter_mut().find(|chan| {
                    chan.start_stop_channel.common_chan_attributes.device_id == old_chan_id
                }) {
                    // Create a new SweepChannel using the device and chan_name
                    let mut new_sweep_channel = SweepChannel::new(
                        sweep_channel
                            .start_stop_channel
                            .common_chan_attributes
                            .chan_name
                            .clone(),
                        device.clone(),
                    );
                    new_sweep_channel
                        .start_stop_channel
                        .common_chan_attributes
                        .uuid = sweep_channel
                        .start_stop_channel
                        .common_chan_attributes
                        .uuid
                        .clone();

                    // Update the existing sweep_channel in the list
                    *sweep_channel = new_sweep_channel;
                }
            }
        }
    }
}
