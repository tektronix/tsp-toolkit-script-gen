use std::{any::Any, collections::HashMap};

use super::function::FunctionModel;
use crate::{
    channel::{
        bias_channel::BiasChannel, default_channel::Channel, step_channel::StepChannel,
        sweep_channel::SweepChannel,
    },
    device_manager::DeviceManager,
    instr_metadata::ki26xx_metadata::Ki26xxMetadata,
};
use script_aggregator::script_buffer::ScriptBuffer;
use xml_handler::group::Group;

#[derive(Debug)]
pub struct SweepModel {
    type_: String,
    description: String,

    auto_range_enabled: bool,
    high_c_enabled: bool,

    metadata: Group,
    val_replacement_map: HashMap<String, String>,

    channels: Vec<Box<dyn Channel>>,
    bias_channels: Vec<BiasChannel>,
    step_channels: Vec<StepChannel>,
    sweep_channels: Vec<SweepChannel>,

    attributes: SweepModelAttributes,
}

impl FunctionModel for SweepModel {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_type(&self) -> &str {
        self.type_.as_str()
    }

    fn get_description(&self) -> &str {
        self.description.as_str()
    }

    fn get_val_replacement_map(&self) -> &std::collections::HashMap<String, String> {
        &self.val_replacement_map
    }

    fn get_metadata(&self) -> &xml_handler::group::Group {
        &self.metadata
    }

    fn to_script(&mut self, script_buffer: &mut ScriptBuffer) {
        if self.step_channels.is_empty() && self.sweep_channels.is_empty() {
            script_buffer.postpend(String::from(
                "-- no sweep ... requires at least 1 step channel or 1 sweep channel",
            ));
        } else {
            //self.val_replacement_map.insert(String::from("DEVICES"), String::from(""));
            self.define_bias_channels();

            self.define_step_channels();

            self.define_sweep_channels();

            self.define_common_settings();

            self.build(script_buffer);
        }
    }
}

impl SweepModel {
    const DESCRIPTION: &'static str = "Configures a sweeping test.";
    const NEGATIVE_NUMBER_NEAR_ZERO: f64 = -1.0e-30;

    pub fn new(group: Group) -> Self {
        SweepModel {
            type_: group.type_.clone(),
            description: Self::DESCRIPTION.to_string(),
            auto_range_enabled: false,
            high_c_enabled: false,
            metadata: group,
            val_replacement_map: HashMap::new(),

            channels: Vec::new(),
            bias_channels: Vec::new(),
            step_channels: Vec::new(),
            sweep_channels: Vec::new(),

            attributes: SweepModelAttributes::new(),
        }
    }

    pub fn auto_configure(&mut self, device_manager: &DeviceManager) {
        //TODO: hardcoding this for now, need to make this dynamic
        let include_step = true;
        self.attributes.step_points = if include_step { 10 } else { 0 };

        let n = device_manager.device_list.len();

        // Prefer the first device (generally smua) for the step -- but if there is only 1 SMU then
        // it must go to a sweep channel (hence the "+ 1")
        if include_step && n > self.channels.len() + 1 {
            self.add_step_channel(StepChannel::new(
                device_manager.get_device(self.channels.len()).clone(),
            ));
        }

        // Assign next available device to sweep channel
        if n > self.channels.len() {
            self.add_sweep_channel(SweepChannel::new(
                device_manager.get_device(self.channels.len()).clone(),
            ));
        }

        // Try to assign next available device to step channel if we need one and haven't got one yet
        if n > self.channels.len() && include_step && self.step_channels.is_empty() {
            self.add_sweep_channel(SweepChannel::new(
                device_manager.get_device(self.channels.len()).clone(),
            ));
        }

        // Assign next available device to bias channel
        if n > self.channels.len() {
            self.add_bias_channel(BiasChannel::new(
                device_manager.get_device(self.channels.len()).clone(),
            ));
        }

        for channel in self.channels.iter_mut() {
            println!("{:?}", channel);
        }
    }

    fn add_bias_channel(&mut self, bias_channel: BiasChannel) {
        self.channels
            .insert(self.bias_channels.len(), Box::new(bias_channel.clone()));
        self.bias_channels.push(bias_channel);
    }

    fn add_step_channel(&mut self, step_channel: StepChannel) {
        self.channels.insert(
            self.bias_channels.len() + self.step_channels.len(),
            Box::new(step_channel.clone()),
        );
        self.step_channels.push(step_channel);
    }

    fn add_sweep_channel(&mut self, sweep_channel: SweepChannel) {
        self.channels.push(Box::new(sweep_channel.clone()));
        self.sweep_channels.push(sweep_channel);
    }

    fn define_step_channels(&mut self) {
        let mut index = 1;
        for step_channel in self.step_channels.iter() {
            //Note - all composite smu stuff will be false, None, 0 for now
            let device = step_channel.get_device();
            let smu = format!("smu{}", index);
            self.attributes.device_names.push(smu.clone());
            self.attributes.step_names.push(smu.clone());

            self.val_replacement_map
                .insert(smu.clone() + ":ASSIGN", device.get_id());
            self.val_replacement_map
                .insert(smu.clone() + ":MODEL", device.get_model());
            self.val_replacement_map
                .insert(smu.clone() + ":VERSION", device.get_fw_version());
            self.val_replacement_map.insert(
                smu.clone() + ":SFUNCTION",
                step_channel.chan_attributes.source_function.clone(),
            );
            self.val_replacement_map.insert(
                smu.clone() + ":SMODE",
                step_channel.chan_attributes.source_mode.clone(),
            );

            self.val_replacement_map
                .insert(smu.clone() + ":COMP-SMU", String::from("false"));
            self.val_replacement_map
                .insert(smu.clone() + ":COMP-SMU-PARALLEL", String::from("false"));
            self.val_replacement_map
                .insert(smu.clone() + ":COMP-SMU-INDEX", String::from("0"));

            self.val_replacement_map
                .insert(smu.clone() + ":SRANGE", String::from("1"));

            let source_limit_i = step_channel.chan_attributes.source_limit_i.abs();
            let source_limit_v = step_channel.chan_attributes.source_limit_v.abs();

            self.val_replacement_map
                .insert(smu.clone() + ":SOURCE-LIMIT-I", self.format(source_limit_i));
            self.val_replacement_map
                .insert(smu.clone() + ":SOURCE-LIMIT-V", self.format(source_limit_v));

            self.val_replacement_map
                .insert(smu.clone() + ":REGION", String::from("1"));
            self.val_replacement_map.insert(
                smu.clone() + ":MFUNCTION",
                step_channel.chan_attributes.measure_function.clone(),
            );
            self.val_replacement_map
                .insert(smu.clone() + ":MRANGE", String::from("0.1"));

            self.val_replacement_map
                .insert(smu.clone() + ":SENSE", String::from("SENSE_LOCAL"));
            self.val_replacement_map.insert(
                smu.clone() + ":MODE",
                step_channel.common_attributes.style.clone(),
            );

            let mut start_value = step_channel.common_attributes.start;
            let mut stop_value = step_channel.common_attributes.stop;
            let asymptote_value = step_channel.common_attributes.asymptote;

            if (start_value <= 0.0 && stop_value <= 0.0)
                && !(start_value == 0.0 && stop_value == 0.0)
            {
                if start_value == 0.0 {
                    start_value = Self::NEGATIVE_NUMBER_NEAR_ZERO;
                }
                if stop_value == 0.0 {
                    stop_value = Self::NEGATIVE_NUMBER_NEAR_ZERO;
                }
            }

            self.val_replacement_map
                .insert(smu.clone() + ":START", self.format(start_value));
            self.val_replacement_map
                .insert(smu.clone() + ":STOP", self.format(stop_value));
            self.val_replacement_map.insert(
                smu.clone() + ":RANGE",
                self.format(stop_value - start_value),
            );
            self.val_replacement_map
                .insert(smu.clone() + ":ASYMPTOTE", self.format(asymptote_value));

            index += 1;
        }

        self.val_replacement_map.insert(
            String::from("STEP-DEVICE"),
            self.comma_separated_list(&self.attributes.step_names),
        );

        self.val_replacement_map.insert(
            String::from("STEP-MASTER"),
            if self.attributes.step_names.is_empty() {
                String::from("")
            } else {
                self.attributes.step_names[0].clone()
            },
        );
        self.val_replacement_map.insert(
            String::from("STEP-COUNT"),
            if self.attributes.step_names.is_empty() {
                String::from("1")
            } else {
                self.attributes.step_points.to_string()
            },
        );
        self.val_replacement_map.insert(
            String::from("STEP-TO-SWEEP-DELAY"),
            if self.attributes.step_names.is_empty() {
                String::from("0")
            } else {
                self.format(self.attributes.step_to_sweep_delay)
            },
        );
    }

    fn define_sweep_channels(&mut self) {
        let master_node = self.deduce_master_node();
        self.val_replacement_map
            .insert(String::from("MASTER-NODE"), master_node.clone());
        let mut index = 1;
        let mut sweep_master: Option<String> = None;
        let mut sweep_master_pulse = String::from("false");
        let mut sweep_nodes: Vec<String> = Vec::new();

        self.val_replacement_map.insert(
            String::from("SWEEP-POINTS"),
            self.attributes.sweep_points.to_string(),
        );

        for sweep_channel in self.sweep_channels.iter() {
            let device = sweep_channel.get_device();
            let smu = format!("smu{}", index);
            self.attributes.device_names.push(smu.clone());
            self.attributes.sweep_names.push(smu.clone());

            self.val_replacement_map
                .insert(smu.clone() + ":ASSIGN", device.get_id());
            self.val_replacement_map
                .insert(smu.clone() + ":MODEL", device.get_model());
            self.val_replacement_map
                .insert(smu.clone() + ":VERSION", device.get_fw_version());
            self.val_replacement_map.insert(
                smu.clone() + ":SFUNCTION",
                sweep_channel.chan_attributes.source_function.clone(),
            );
            self.val_replacement_map.insert(
                smu.clone() + ":SMODE",
                sweep_channel.chan_attributes.source_mode.clone(),
            );

            self.val_replacement_map
                .insert(smu.clone() + ":COMP-SMU", String::from("false"));
            self.val_replacement_map
                .insert(smu.clone() + ":COMP-SMU-PARALLEL", String::from("false"));
            self.val_replacement_map
                .insert(smu.clone() + ":COMP-SMU-INDEX", String::from("0"));

            self.val_replacement_map
                .insert(smu.clone() + ":SRANGE", String::from("1"));

            let source_limit_i = sweep_channel.chan_attributes.source_limit_i.abs();
            let source_limit_v = sweep_channel.chan_attributes.source_limit_v.abs();

            if sweep_channel.chan_attributes.pulse_enabled {
                todo!();
            } else {
                self.val_replacement_map
                    .insert(smu.clone() + ":SOURCE-LIMIT-I", self.format(source_limit_i));
                self.val_replacement_map
                    .insert(smu.clone() + ":SOURCE-LIMIT-V", self.format(source_limit_i));
                self.val_replacement_map
                    .insert(smu.clone() + ":TRIGLIMITI", String::from("0.0"));
                self.val_replacement_map
                    .insert(smu.clone() + ":TRIGLIMITV", String::from("0.0"));
            }

            self.val_replacement_map
                .insert(smu.clone() + ":REGION", String::from("1"));
            self.val_replacement_map.insert(
                smu.clone() + ":MFUNCTION",
                sweep_channel.chan_attributes.measure_function.clone(),
            );
            self.val_replacement_map
                .insert(smu.clone() + ":MRANGE", String::from("0.1"));

            self.val_replacement_map
                .insert(smu.clone() + ":SENSE", String::from("SENSE_LOCAL"));

            let mode = if self.attributes.custom_sweep {
                String::from("LIST")
            } else {
                sweep_channel.common_attributes.style.clone()
            };
            self.val_replacement_map.insert(smu.clone() + ":MODE", mode);

            let mut start_value = sweep_channel.common_attributes.start;
            let mut stop_value = sweep_channel.common_attributes.stop;
            let asymptote_value = sweep_channel.common_attributes.asymptote;
            let mut pulse_bias_value = sweep_channel.pulse_bias;

            if self.attributes.custom_sweep {
                todo!();
            }

            let mut is_negative_going = false;
            if self.attributes.custom_sweep {
                todo!();
            } else {
                if start_value < 0.0 || stop_value < 0.0 {
                    is_negative_going = true;
                }
            }

            if sweep_channel.chan_attributes.pulse_enabled && pulse_bias_value < 0.0 {
                is_negative_going = true;
            }

            if is_negative_going {
                if self.attributes.custom_sweep {
                    todo!();
                } else if start_value > 0.0 || stop_value > 0.0 {
                    is_negative_going = false;
                }

                if sweep_channel.chan_attributes.pulse_enabled && pulse_bias_value > 0.0 {
                    is_negative_going = false;
                }

                if is_negative_going {
                    if self.attributes.custom_sweep {
                        todo!();
                    } else {
                        if start_value == 0.0 {
                            start_value = Self::NEGATIVE_NUMBER_NEAR_ZERO;
                        }
                        if stop_value == 0.0 {
                            stop_value = Self::NEGATIVE_NUMBER_NEAR_ZERO;
                        }
                    }
                    if sweep_channel.chan_attributes.pulse_enabled && pulse_bias_value == 0.0 {
                        pulse_bias_value = Self::NEGATIVE_NUMBER_NEAR_ZERO;
                    }
                }
            }

            self.val_replacement_map
                .insert(smu.clone() + ":START", self.format(start_value));
            self.val_replacement_map
                .insert(smu.clone() + ":STOP", self.format(stop_value));
            self.val_replacement_map.insert(
                smu.clone() + ":RANGE",
                self.format(stop_value - start_value),
            );
            let local_pulse_mode = if sweep_channel.chan_attributes.pulse_enabled {
                String::from("true")
            } else {
                String::from("false")
            };
            self.val_replacement_map
                .insert(smu.clone() + ":PULSE-MODE", local_pulse_mode);
            self.val_replacement_map
                .insert(smu.clone() + ":PULSE-BIAS", self.format(pulse_bias_value));
            self.val_replacement_map
                .insert(smu.clone() + ":ASYMPTOTE", self.format(asymptote_value));

            self.val_replacement_map
                .insert(smu.clone() + ":LIST", String::from("0"));

            let node = device.get_node_id();
            if sweep_master.is_none() && master_node == node {
                sweep_master = Some(smu);
                sweep_master_pulse = if sweep_channel.chan_attributes.pulse_enabled {
                    String::from("true")
                } else {
                    String::from("false")
                };
            }

            if !sweep_nodes.contains(&node) {
                sweep_nodes.push(node);
            }
            index += 1;
        }

        let mut bias_nodes: Vec<String> = Vec::new();
        for bias_channel in self.bias_channels.iter() {
            let node = bias_channel.get_device().get_node_id();
            if !bias_nodes.contains(&node) {
                bias_nodes.push(node);
            }
        }

        self.val_replacement_map.insert(
            String::from("SWEEP-DEVICE"),
            self.comma_separated_list(&self.attributes.sweep_names),
        );

        let sweep_smus = self.format_smus(&sweep_nodes, &self.sweep_channels);

        self.val_replacement_map.insert(
            String::from("SWEEP-MASTER"),
            if let Some(sweep_master) = sweep_master {
                sweep_master
            } else {
                String::from("")
            },
        );
        self.val_replacement_map
            .insert(String::from("SWEEP-NODES"), self.format_list(&sweep_nodes));
        self.val_replacement_map.insert(
            String::from("SWEEP-SMUS"),
            if sweep_smus.is_empty() {
                String::from("{}")
            } else {
                sweep_smus
            },
        );
        self.val_replacement_map.insert(
            String::from("SWEEP-PULSE"),
            self.format_pulse(&sweep_nodes, &self.sweep_channels),
        );
        self.val_replacement_map
            .insert(String::from("SWEEP-MASTER-PULSE"), sweep_master_pulse);

        // Also need a variant of sweepNodes ... without the master node
        sweep_nodes.retain(|x| x != &master_node);
        self.val_replacement_map.insert(
            String::from("SWEEP-SLAVE-NODES"),
            self.inner_format_list(&sweep_nodes, true),
        );
        self.val_replacement_map.insert(
            String::from("SWEEP-SLAVE-PULSE"),
            self.inner_format_pulse(&sweep_nodes, &self.sweep_channels, true),
        );
        self.val_replacement_map.insert(
            String::from("SWEEP-SLAVE-SMUS"),
            self.inner_format_smus(&sweep_nodes, &self.sweep_channels, true),
        );

        let bias_smus = self.format_bias_smus(&bias_nodes, &self.bias_channels, false);
        self.val_replacement_map.insert(
            String::from("BIAS-NODES"),
            self.inner_format_list(&bias_nodes, false),
        );
        self.val_replacement_map.insert(
            String::from("BIAS-SMUS"),
            if bias_smus.is_empty() {
                String::from("{}")
            } else {
                bias_smus
            },
        );
    }

    fn define_bias_channels(&mut self) {
        let mut index = 1;
        for bias_channel in self.bias_channels.iter() {
            let device = bias_channel.get_device();
            let smu = format!("bias{}", index);
            self.attributes.bias_names.push(smu.clone());

            self.val_replacement_map
                .insert(smu.clone() + ":ASSIGN", device.get_id());
            self.val_replacement_map
                .insert(smu.clone() + ":MODEL", device.get_model());
            self.val_replacement_map
                .insert(smu.clone() + ":VERSION", device.get_fw_version());
            self.val_replacement_map.insert(
                smu.clone() + ":SFUNCTION",
                bias_channel.chan_attributes.source_function.clone(),
            );
            self.val_replacement_map.insert(
                smu.clone() + ":SMODE",
                bias_channel.chan_attributes.source_mode.clone(),
            );

            self.val_replacement_map
                .insert(smu.clone() + ":COMP-SMU", String::from("false"));
            self.val_replacement_map
                .insert(smu.clone() + ":COMP-SMU-PARALLEL", String::from("false"));
            self.val_replacement_map
                .insert(smu.clone() + ":COMP-SMU-INDEX", String::from("0"));

            self.val_replacement_map
                .insert(smu.clone() + ":SRANGE", String::from("1"));

            let source_limit_i = bias_channel.chan_attributes.source_limit_i.abs();
            let source_limit_v = bias_channel.chan_attributes.source_limit_v.abs();

            self.val_replacement_map
                .insert(smu.clone() + ":SOURCE-LIMIT-I", self.format(source_limit_i));
            self.val_replacement_map
                .insert(smu.clone() + ":SOURCE-LIMIT-V", self.format(source_limit_v));

            self.val_replacement_map
                .insert(smu.clone() + ":REGION", String::from("1"));
            self.val_replacement_map.insert(
                smu.clone() + ":MFUNCTION",
                bias_channel.chan_attributes.measure_function.clone(),
            );

            self.val_replacement_map
                .insert(smu.clone() + ":MRANGE", String::from("0.1"));

            self.val_replacement_map
                .insert(smu.clone() + ":SENSE", String::from("SENSE_LOCAL"));

            self.val_replacement_map
                .insert(smu.clone() + ":BIAS", self.format(0.0));

            index += 1;
        }

        self.val_replacement_map.insert(
            String::from("BIAS-DEVICE"),
            self.comma_separated_list(&self.attributes.bias_names),
        );
    }

    fn define_common_settings(&mut self) {
        // Extend device_names with step_names and sweep_names
        self.attributes
            .device_names
            .extend(self.attributes.bias_names.iter().cloned());
        self.attributes
            .device_names
            .extend(self.attributes.step_names.iter().cloned());
        self.attributes
            .device_names
            .extend(self.attributes.step_names.iter().cloned());

        self.val_replacement_map.insert(
            String::from("DEVICES"),
            self.comma_separated_list(&self.attributes.device_names),
        );

        const TRIGGER_MODELS: [&str; 16] = [
            "unknown",
            "local-step",
            "unknown",
            "unknown",
            "local-sweep",
            "local",
            "unknown",
            "unknown",
            "unknown",
            "unknown",
            "unknown",
            "unknown",
            "unknown",
            "unknown",
            "unknown",
            "unknown",
        ];

        let mut temp = 0;
        for step_channel in self.step_channels.iter() {
            let device_id = step_channel.get_device().get_id();
            temp |= if device_id.starts_with("local") { 1 } else { 2 };
        }

        for sweep_channel in self.sweep_channels.iter() {
            let device_id = sweep_channel.get_device().get_id();
            temp |= if device_id.starts_with("local") { 4 } else { 8 };
        }
        self.val_replacement_map.insert(
            String::from("TRIGGER-MODEL"),
            TRIGGER_MODELS[temp].to_string(),
        );

        // Add the global values to the environment ... any device should be suitable for
        // display->device conversions
        let device = self.channels[0].get_device();
        self.val_replacement_map.insert(
            String::from("EPSILON"),
            self.format(Ki26xxMetadata::EPSILON),
        );
        self.val_replacement_map.insert(
            String::from("LINE-FREQUENCY"),
            self.attributes.line_frequency.to_string(),
        );
        self.val_replacement_map.insert(
            String::from("BIAS-CHANNEL-COUNT"),
            self.bias_channels.len().to_string(),
        );
        self.val_replacement_map.insert(
            String::from("STEP-CHANNEL-COUNT"),
            self.step_channels.len().to_string(),
        );
        self.val_replacement_map.insert(
            String::from("SWEEP-CHANNEL-COUNT"),
            self.sweep_channels.len().to_string(),
        );
        self.val_replacement_map.insert(
            String::from("AUTORANGE-ENABLED"),
            self.attributes.auto_range_enabled.to_string(),
        );

        //TODO - more replacements
    }

    fn deduce_master_node(&self) -> String {
        let mut master_node = String::from("NoMasterNode");
        if !self.step_channels.is_empty() {
            master_node = self.step_channels[0].get_device().get_node_id();
        } else if !self.sweep_channels.is_empty() {
            master_node = self.sweep_channels[0].get_device().get_node_id()
        }
        master_node
    }

    fn format_smus(&self, list: &Vec<String>, available: &Vec<SweepChannel>) -> String {
        self.inner_format_smus(list, available, false)
    }

    fn inner_format_smus(
        &self,
        list: &Vec<String>,
        available: &Vec<SweepChannel>,
        prepend_comma: bool,
    ) -> String {
        let mut buffer = String::new();
        for node in list {
            if !buffer.is_empty() {
                buffer.push_str(", ");
            }
            buffer.push_str("{ ");
            for chan in available {
                if chan.get_device().get_node_id() == *node {
                    buffer.push_str(&chan.get_device().get_id());
                }
            }
            buffer.push_str(" }");
        }
        if !buffer.is_empty() && prepend_comma {
            buffer.insert_str(0, ", ");
        }
        buffer
    }

    fn format_bias_smus(
        &self,
        list: &Vec<String>,
        available: &Vec<BiasChannel>,
        prepend_comma: bool,
    ) -> String {
        let mut buffer = String::new();
        for node in list {
            if !buffer.is_empty() {
                buffer.push_str(", ");
            }
            buffer.push_str("{ ");
            for chan in available {
                if chan.get_device().get_node_id() == *node {
                    buffer.push_str(&chan.get_device().get_id());
                }
            }
            buffer.push_str(" }");
        }
        if !buffer.is_empty() && prepend_comma {
            buffer.insert_str(0, ", ");
        }
        buffer
    }

    fn format_pulse(&self, list: &Vec<String>, available: &Vec<SweepChannel>) -> String {
        self.inner_format_pulse(list, available, false)
    }

    fn inner_format_pulse(
        &self,
        list: &Vec<String>,
        available: &Vec<SweepChannel>,
        prepend_comma: bool,
    ) -> String {
        let mut buffer = String::new();
        for node in list {
            if !buffer.is_empty() {
                buffer.push_str(", ");
            }
            buffer.push_str("{ ");
            for chan in available {
                if chan.get_device().get_node_id() == *node {
                    let res = if chan.chan_attributes.pulse_enabled {
                        String::from("true")
                    } else {
                        String::from("false")
                    };
                    buffer.push_str(&res);
                }
            }
            buffer.push_str(" }");
        }
        if !buffer.is_empty() && prepend_comma {
            buffer.insert_str(0, ", ");
        }
        buffer
    }

    fn format_list(&self, list: &Vec<String>) -> String {
        self.inner_format_list(list, false)
    }

    fn inner_format_list(&self, list: &Vec<String>, prepend_comma: bool) -> String {
        let mut buffer = String::with_capacity(34);
        let mut num_chars = 0;

        for value in list {
            if !buffer.is_empty() {
                buffer.push_str(", ");
                num_chars += 2;

                // The input buffer for TSP command interface can only process up to 1000
                // characters per line -> insert a new line character every ~500 chars.
                if num_chars > 500 {
                    buffer.push('\n');
                    num_chars = 0;
                }
            }
            buffer.push_str(value);
            num_chars += value.len();
        }

        if !buffer.is_empty() && prepend_comma {
            buffer.insert_str(0, ", ");
        }

        buffer
    }

    fn comma_separated_list(&self, list: &Vec<String>) -> String {
        let mut buffer = String::new();
        for value in list {
            if !buffer.is_empty() {
                buffer.push(',');
            }
            buffer.push_str(value);
        }
        buffer
    }
}

#[derive(Debug)]
pub struct SweepModelAttributes {
    device_names: Vec<String>,
    step_names: Vec<String>,
    sweep_names: Vec<String>,
    bias_names: Vec<String>,

    sweep_points: i32,
    custom_sweep: bool,
    step_points: i32,
    step_to_sweep_delay: f64,
    line_frequency: i32,
    auto_range_enabled: bool,
}

impl SweepModelAttributes {
    pub fn new() -> Self {
        SweepModelAttributes {
            device_names: Vec::new(),
            step_names: Vec::new(),
            sweep_names: Vec::new(),
            bias_names: Vec::new(),

            sweep_points: 10,
            custom_sweep: false,
            step_points: 10,
            step_to_sweep_delay: 0.0,
            line_frequency: 60,
            auto_range_enabled: false,
        }
    }
}
