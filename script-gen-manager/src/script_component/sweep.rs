use std::{any::Any, collections::HashMap};

use crate::{
    device::DeviceType,
    instr_metadata::base_metadata::BaseMetadata,
    model::{
        chan_data::channel_range::ChannelRange,
        sweep_data::{
            parameters::{ParameterFloat, ParameterString},
            sweep_config::SweepConfig,
        },
    },
};

use super::function::FunctionModel;
use script_aggregator::script_buffer::ScriptBuffer;
use xml_handler::group::Group;

/// SweepModel is an aggregation of FunctionModel that represents the _Sweep() function of the script.
/// This function executes a nested step/sweep operation and is optional in the generated script.
#[derive(Debug)]
pub struct SweepModel {
    type_: String,
    description: String,

    metadata: Group,
    val_replacement_map: HashMap<String, String>,

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

    fn to_script(&mut self, sweep_config: &SweepConfig, script_buffer: &mut ScriptBuffer) {
        if sweep_config.step_channels.is_empty() && sweep_config.sweep_channels.is_empty() {
            script_buffer.postamble_append(String::from(
                "-- no sweep ... requires at least 1 step channel or 1 sweep channel",
            ));
        } else {
            self.attributes = SweepModelAttributes::new();
            self.val_replacement_map = HashMap::new();

            self.define_bias_channels(sweep_config);
            self.define_step_channels(sweep_config);
            self.define_sweep_channels(sweep_config);
            self.define_common_settings(sweep_config);
            self.build(script_buffer);
        }
    }
}

impl SweepModel {
    const DESCRIPTION: &'static str = "Configures a sweeping test.";
    // const NEGATIVE_NUMBER_NEAR_ZERO: f64 = -1.0e-30;

    pub fn new(group: Group) -> Self {
        SweepModel {
            type_: group.type_.clone(),
            description: Self::DESCRIPTION.to_string(),
            metadata: group,
            val_replacement_map: HashMap::new(),

            attributes: SweepModelAttributes::new(),
        }
    }

    fn define_bias_channels(&mut self, bias_config: &SweepConfig) {
        let mut index = 1;
        for bias_channel in bias_config.bias_channels.iter() {
            let instr_name = format!("bias{index}");
            self.attributes.bias_names.push(instr_name.clone());

            self.val_replacement_map.insert(
                instr_name.clone() + ":NODE",
                bias_channel.common_chan_attributes.device.get_node_id(),
            );

            if let Some((node_idx, slot_idx, channel_idx)) =
                self.extract_indices(&bias_channel.common_chan_attributes.device.get_id())
            {
                self.val_replacement_map
                    .insert(instr_name.clone() + ":NODE-IDX", node_idx.to_string());
                self.val_replacement_map
                    .insert(instr_name.clone() + ":SLOT-IDX", slot_idx.to_string());
                self.val_replacement_map
                    .insert(instr_name.clone() + ":CHANNEL-IDX", channel_idx.to_string());
            }
            self.val_replacement_map.insert(
                instr_name.clone() + ":MODEL",
                bias_channel.common_chan_attributes.device.get_model(),
            );

            self.val_replacement_map.insert(
                instr_name.clone() + ":MODEL-TYPE",
                bias_channel
                    .common_chan_attributes
                    .device
                    .device_type
                    .to_string(),
            );

            let val = self
                .get_function_value(&bias_channel.common_chan_attributes.source_function)
                .clone();
            self.val_replacement_map
                .insert(instr_name.clone() + ":SFUNCTION", val);

            self.set_source_range(
                bias_channel.common_chan_attributes.source_range.clone(),
                &instr_name,
            );

            self.set_measure_range(
                bias_channel.common_chan_attributes.meas_range.clone(),
                &instr_name,
            );

            let val = self
                .get_function_value(&bias_channel.common_chan_attributes.meas_function)
                .clone();

            self.val_replacement_map
                .insert(instr_name.clone() + ":MFUNCTION", val);

            //sense mode exists only for SMU
            if let Some(sense_mode) = &bias_channel.common_chan_attributes.sense_mode {
                let sense_mode_key = format!("sense={}", sense_mode.value);
                if let Some(sense_mode_value) = bias_channel
                    .common_chan_attributes
                    .get_name_for(&sense_mode_key)
                {
                    self.val_replacement_map
                        .insert(instr_name.clone() + ":SENSE", sense_mode_value.to_owned());
                }
            } else {
                self.val_replacement_map.insert(
                    instr_name.clone() + ":SENSE",
                    BaseMetadata::UNDEFINED.to_string(),
                );
            }

            //source_limitv exists only for SMU
            if let Some(source_limitv) = &bias_channel.common_chan_attributes.source_limitv {
                self.val_replacement_map.insert(
                    instr_name.clone() + ":LIMITV",
                    self.format(source_limitv.value),
                );
            } else {
                self.val_replacement_map
                    .insert(instr_name.clone() + ":LIMITV", String::from("nil"));
            }

            if let Some(source_limiti) = &bias_channel.common_chan_attributes.source_limiti {
                self.val_replacement_map.insert(
                    instr_name.clone() + ":LIMITI",
                    self.format(source_limiti.value),
                );
            } else {
                self.val_replacement_map
                    .insert(instr_name.clone() + ":LIMITI", String::from("nil"));
            }

            self.val_replacement_map.insert(
                instr_name.clone() + ":BIAS",
                self.format(bias_channel.bias.value),
            );

            index += 1;
        }

        if !self.attributes.bias_names.is_empty() {
            self.val_replacement_map.insert(
                String::from("BIAS-DEVICE"),
                self.comma_separated_list(&self.attributes.bias_names),
            );
        }
    }

    fn set_source_range(&mut self, channel_range: ChannelRange, instr_name: &String) {
        let mut val = self.format_range(channel_range.clone());

        if channel_range.is_range_auto() {
            val = "CONSTANTS.AUTO".to_string();
        }

        self.val_replacement_map
            .insert(instr_name.clone() + ":SRANGE", val);
    }

    fn set_measure_range(&mut self, channel_range: ChannelRange, instr_name: &String) {
        let mut val = self.format_range(channel_range.clone());

        if channel_range.is_range_auto() {
            val = "CONSTANTS.AUTO".to_string();
        }
        self.val_replacement_map
            .insert(instr_name.clone() + ":MRANGE", val);
    }

    //Returns the value used in the script
    fn get_function_value(&mut self, source_function: &ParameterString) -> String {
        if source_function.value.to_lowercase() == BaseMetadata::FUNCTION_VOLTAGE.to_lowercase() {
            "FUNC_DC_VOLTAGE".to_string()
        } else if source_function.value.to_lowercase()
            == BaseMetadata::FUNCTION_CURRENT.to_lowercase()
        {
            "FUNC_DC_CURRENT".to_string()
        } else {
            "FUNC_DC_IV_COMBINED".to_string()
        }
    }

    fn define_step_channels(&mut self, step_config: &SweepConfig) {
        let mut index = 1;
        for step_channel in step_config.step_channels.iter() {
            let instr_name = format!("step{index}");
            self.attributes.step_names.push(instr_name.clone());

            self.val_replacement_map.insert(
                instr_name.clone() + ":NODE",
                step_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device
                    .get_node_id(),
            );

            if let Some((node_idx, slot_idx, channel_idx)) = self.extract_indices(
                &step_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device
                    .get_id(),
            ) {
                self.val_replacement_map
                    .insert(instr_name.clone() + ":NODE-IDX", node_idx.to_string());
                self.val_replacement_map
                    .insert(instr_name.clone() + ":SLOT-IDX", slot_idx.to_string());
                self.val_replacement_map
                    .insert(instr_name.clone() + ":CHANNEL-IDX", channel_idx.to_string());
            }
            self.val_replacement_map.insert(
                instr_name.clone() + ":MODEL",
                step_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device
                    .get_model(),
            );

            self.val_replacement_map.insert(
                instr_name.clone() + ":MODEL-TYPE",
                step_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device
                    .device_type
                    .to_string(),
            );

            let val = self
                .get_function_value(
                    &step_channel
                        .start_stop_channel
                        .common_chan_attributes
                        .source_function,
                )
                .clone();
            self.val_replacement_map
                .insert(instr_name.clone() + ":SFUNCTION", val);

            self.set_source_range(
                step_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .source_range
                    .clone(),
                &instr_name,
            );

            self.set_measure_range(
                step_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .meas_range
                    .clone(),
                &instr_name,
            );

            let val = self
                .get_function_value(
                    &step_channel
                        .start_stop_channel
                        .common_chan_attributes
                        .meas_function,
                )
                .clone();
            self.val_replacement_map
                .insert(instr_name.clone() + ":MFUNCTION", val);

            self.val_replacement_map.insert(
                instr_name.clone() + ":MODE",
                if step_config.step_global_parameters.list_step {
                    "LIST".to_string()
                } else {
                    step_channel.start_stop_channel.style.value.clone()
                },
            );

            //sense mode exists only for SMU
            if let Some(sense_mode) = &step_channel
                .start_stop_channel
                .common_chan_attributes
                .sense_mode
            {
                let sense_mode_key = format!("sense={}", sense_mode.value);
                if let Some(sense_mode_value) = step_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .get_name_for(&sense_mode_key)
                {
                    self.val_replacement_map
                        .insert(instr_name.clone() + ":SENSE", sense_mode_value.to_owned());
                }
            } else {
                self.val_replacement_map.insert(
                    instr_name.clone() + ":SENSE",
                    BaseMetadata::UNDEFINED.to_string(),
                );
            }

            //source_limitv exists only for SMU
            if let Some(source_limitv) = &step_channel
                .start_stop_channel
                .common_chan_attributes
                .source_limitv
            {
                self.val_replacement_map.insert(
                    instr_name.clone() + ":LIMITV",
                    self.format(source_limitv.value),
                );
            } else {
                self.val_replacement_map
                    .insert(instr_name.clone() + ":LIMITV", String::from("nil"));
            }

            if let Some(source_limiti) = &step_channel
                .start_stop_channel
                .common_chan_attributes
                .source_limiti
            {
                self.val_replacement_map.insert(
                    instr_name.clone() + ":LIMITI",
                    self.format(source_limiti.value),
                );
            } else {
                self.val_replacement_map
                    .insert(instr_name.clone() + ":LIMITI", String::from("nil"));
            }

            self.val_replacement_map.insert(
                instr_name.clone() + ":START",
                self.format(step_channel.start_stop_channel.start.value),
            );
            self.val_replacement_map.insert(
                instr_name.clone() + ":STOP",
                self.format(step_channel.start_stop_channel.stop.value),
            );

            self.process_list(
                step_config.step_global_parameters.list_step,
                &step_channel.start_stop_channel.list,
                instr_name,
                step_config.step_global_parameters.step_points.value as usize,
            );

            index += 1;
        }

        let step_count = if !self.attributes.step_names.is_empty() {
            step_config
                .step_global_parameters
                .step_points
                .value
                .to_string()
        } else {
            String::from("1")
        };

        let step_to_sweep_delay = if !self.attributes.step_names.is_empty() {
            self.format(step_config.step_global_parameters.step_to_sweep_delay.value)
        } else {
            String::from("0")
        };

        self.val_replacement_map
            .insert(String::from("STEP-COUNT"), step_count);
        self.val_replacement_map
            .insert(String::from("STEP-TO-SWEEP-DELAY"), step_to_sweep_delay);

        if !self.attributes.step_names.is_empty() {
            self.val_replacement_map.insert(
                String::from("STEP-DEVICE"),
                self.comma_separated_list(&self.attributes.step_names),
            );
        }
    }

    fn process_list(
        &mut self,
        is_list: bool,
        list: &Vec<ParameterFloat>,
        instr_name: String,
        len: usize, // Default length for list values
    ) {
        //Default value for list is nil
        let mut list_values = "nil".to_string();

        if is_list {
            let mut new_list = list
                .iter()
                .map(|item| item.value.to_string())
                .collect::<Vec<_>>();
            if !new_list.is_empty() && new_list.len() > len {
                new_list.truncate(len);
            }

            //Fill in list values
            list_values = format!("{{ {} }}", new_list.join(", "));
        }
        self.val_replacement_map
            .insert(instr_name.clone() + ":LIST", list_values);
    }

    fn define_sweep_channels(&mut self, sweep_config: &SweepConfig) {
        let mut index = 1;
        for sweep_channel in sweep_config.sweep_channels.iter() {
            let instr_name = format!("sweep{index}");
            self.attributes.sweep_names.push(instr_name.clone());

            self.val_replacement_map.insert(
                instr_name.clone() + ":NODE",
                sweep_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device
                    .get_node_id(),
            );

            if let Some((node_idx, slot_idx, channel_idx)) = self.extract_indices(
                &sweep_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device
                    .get_id(),
            ) {
                self.val_replacement_map
                    .insert(instr_name.clone() + ":NODE-IDX", node_idx.to_string());
                self.val_replacement_map
                    .insert(instr_name.clone() + ":SLOT-IDX", slot_idx.to_string());
                self.val_replacement_map
                    .insert(instr_name.clone() + ":CHANNEL-IDX", channel_idx.to_string());
            }
            self.val_replacement_map.insert(
                instr_name.clone() + ":MODEL",
                sweep_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device
                    .get_model(),
            );

            self.val_replacement_map.insert(
                instr_name.clone() + ":MODEL-TYPE",
                sweep_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .device
                    .device_type
                    .to_string(),
            );

            let val = self
                .get_function_value(
                    &sweep_channel
                        .start_stop_channel
                        .common_chan_attributes
                        .source_function,
                )
                .clone();
            self.val_replacement_map
                .insert(instr_name.clone() + ":SFUNCTION", val);

            self.set_source_range(
                sweep_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .source_range
                    .clone(),
                &instr_name,
            );

            self.set_measure_range(
                sweep_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .meas_range
                    .clone(),
                &instr_name,
            );

            let val = self
                .get_function_value(
                    &sweep_channel
                        .start_stop_channel
                        .common_chan_attributes
                        .meas_function,
                )
                .clone();
            self.val_replacement_map
                .insert(instr_name.clone() + ":MFUNCTION", val);

            self.val_replacement_map.insert(
                instr_name.clone() + ":MODE",
                if sweep_config.sweep_global_parameters.list_sweep {
                    "LIST".to_string()
                } else {
                    sweep_channel.start_stop_channel.style.value.clone()
                },
            );
            //sense mode exists only for SMU
            if let Some(sense_mode) = &sweep_channel
                .start_stop_channel
                .common_chan_attributes
                .sense_mode
            {
                let sense_mode_key = format!("sense={}", sense_mode.value);
                if let Some(sense_mode_value) = sweep_channel
                    .start_stop_channel
                    .common_chan_attributes
                    .get_name_for(&sense_mode_key)
                {
                    self.val_replacement_map
                        .insert(instr_name.clone() + ":SENSE", sense_mode_value.to_owned());
                }
            } else {
                self.val_replacement_map.insert(
                    instr_name.clone() + ":SENSE",
                    BaseMetadata::UNDEFINED.to_string(),
                );
            }

            //source_limitv exists only for SMU
            if let Some(source_limitv) = &sweep_channel
                .start_stop_channel
                .common_chan_attributes
                .source_limitv
            {
                self.val_replacement_map.insert(
                    instr_name.clone() + ":LIMITV",
                    self.format(source_limitv.value),
                );
            } else {
                self.val_replacement_map
                    .insert(instr_name.clone() + ":LIMITV", String::from("nil"));
            }

            if let Some(source_limiti) = &sweep_channel
                .start_stop_channel
                .common_chan_attributes
                .source_limiti
            {
                self.val_replacement_map.insert(
                    instr_name.clone() + ":LIMITI",
                    self.format(source_limiti.value),
                );
            } else {
                self.val_replacement_map
                    .insert(instr_name.clone() + ":LIMITI", String::from("nil"));
            }

            self.val_replacement_map.insert(
                instr_name.clone() + ":START",
                self.format(sweep_channel.start_stop_channel.start.value),
            );
            self.val_replacement_map.insert(
                instr_name.clone() + ":STOP",
                self.format(sweep_channel.start_stop_channel.stop.value),
            );

            self.process_list(
                sweep_config.sweep_global_parameters.list_sweep,
                &sweep_channel.start_stop_channel.list,
                instr_name,
                sweep_config.sweep_global_parameters.sweep_points.value as usize,
            );

            index += 1;
        }

        self.val_replacement_map.insert(
            String::from("SWEEP-POINTS"),
            sweep_config
                .sweep_global_parameters
                .sweep_points
                .value
                .to_string(),
        );
        if !self.attributes.sweep_names.is_empty() {
            self.val_replacement_map.insert(
                String::from("SWEEP-DEVICE"),
                self.comma_separated_list(&self.attributes.sweep_names),
            );
        }
    }

    fn define_common_settings(&mut self, sweep_config: &SweepConfig) {
        if sweep_config
            .global_parameters
            .sweep_timing_config
            .smu_timing
            .nplc_type
            .value
            == "NPLC"
        {
            self.val_replacement_map.insert(
                String::from("NPLC"),
                self.format(
                    sweep_config
                        .global_parameters
                        .sweep_timing_config
                        .smu_timing
                        .nplc
                        .value,
                ),
            );
            self.val_replacement_map
                .insert(String::from("APERTURE"), "nil".to_owned());
        } else {
            self.val_replacement_map.insert(
                String::from("APERTURE"),
                self.format(
                    sweep_config
                        .global_parameters
                        .sweep_timing_config
                        .smu_timing
                        .aperture
                        .value,
                ),
            );
            self.val_replacement_map
                .insert(String::from("NPLC"), "nil".to_owned());
        }

        self.val_replacement_map.insert(
            String::from("MEASURE-COUNT"),
            self.format(
                sweep_config
                    .global_parameters
                    .sweep_timing_config
                    .measure_count
                    .value
                    .into(),
            ),
        );

        if sweep_config
            .global_parameters
            .sweep_timing_config
            .smu_timing
            .measure_auto_delay
            .value
            == BaseMetadata::OFF_VALUE
        {
            self.val_replacement_map.insert(
                String::from("MEASURE-DELAY"),
                self.format(
                    sweep_config
                        .global_parameters
                        .sweep_timing_config
                        .smu_timing
                        .measure_delay
                        .value,
                ),
            );
        } else {
            self.val_replacement_map
                .insert(String::from("MEASURE-DELAY"), String::from("nil"));
        }

        if sweep_config
            .global_parameters
            .sweep_timing_config
            .smu_timing
            .source_auto_delay
            .value
            == BaseMetadata::OFF_VALUE
        {
            self.val_replacement_map.insert(
                String::from("SOURCE-DELAY"),
                self.format(
                    sweep_config
                        .global_parameters
                        .sweep_timing_config
                        .smu_timing
                        .source_delay
                        .value,
                ),
            );
        } else {
            self.val_replacement_map
                .insert(String::from("SOURCE-DELAY"), String::from("nil"));
        }
    }

    /// Builds a string representing comma-separated list of values from the provided list.
    ///
    /// # Arguments
    /// * `list` - A reference to a vector of strings.
    ///
    /// # Returns
    ///
    /// A formatted string representing the list of values.
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

    /// Formats the value of a `ChannelRange` object into a string.
    ///
    /// # Arguments
    /// * `range` - A `ChannelRange` object containing the range value.
    ///
    /// # Returns
    /// A formatted string representing the range value. If the range is set to auto or follow limit,
    /// the original value is returned. Otherwise, the scaled value is formatted with 3 decimal places.
    fn format_range(&self, range: ChannelRange) -> String {
        let mut result = String::from("NaN");
        if range.is_range_auto() || range.is_range_follow_limiti() {
            result = range.value;
        } else {
            let range_value = range.get_scaled_value();
            if let Some(value) = range_value {
                // Format with 3 decimal places, using scientific notation if needed
                if value.abs() < 1e-3 || value.abs() >= 1e3 {
                    result = format!("{:.3e}", value);
                } else {
                    result = format!("{:.3}", value);
                }
            }
        }
        result
    }

    /// Extracts numeric indices from a formatted string containing bracketed numbers.
    ///
    /// # Arguments
    /// * `input` - A string slice containing the input string, such as `"node[37].slot[1].smu[1]"`
    ///   or `"localnode.slot[1].smu[1]"`.
    ///
    /// # Returns
    /// An `Option` containing a tuple `(node_idx, slot_idx, channel_idx)`:
    /// - `node_idx`: The number inside the `node[...]` part, or `0` if the input starts with `"localnode"`.
    /// - `slot_idx`: The number inside the `slot[...]` part.
    /// - `channel_idx`: The number inside the last part (e.g., `smu[...]` or `psu[...]`).
    ///
    /// Returns `None` if the input string does not contain exactly three bracketed numbers.
    ///
    /// # Behavior
    /// - If the input starts with `"localnode"`, `node_idx` is set to `0`.
    /// - Iterates through the input string and extracts numeric values enclosed in square brackets (`[...]`).
    /// - Parses the extracted numbers into `usize` values.
    /// - Ensures that exactly three numbers are found; otherwise, returns `None`.
    ///
    /// # Example
    /// ```rust
    /// let input1 = "node[37].slot[1].smu[1]";
    /// if let Some((node_idx, slot_idx, channel_idx)) = sweep_model.extract_indices(input1) {
    ///     println!("Node Index: {}", node_idx); // Outputs: 37
    ///     println!("Slot Index: {}", slot_idx); // Outputs: 1
    ///     println!("Channel Index: {}", channel_idx); // Outputs: 1
    /// }
    ///
    /// let input2 = "localnode.slot[1].smu[1]";
    /// if let Some((node_idx, slot_idx, channel_idx)) = sweep_model.extract_indices(input2) {
    ///     println!("Node Index: {}", node_idx); // Outputs: 0
    ///     println!("Slot Index: {}", slot_idx); // Outputs: 1
    ///     println!("Channel Index: {}", channel_idx); // Outputs: 1
    /// }
    /// ```
    fn extract_indices(&self, input: &str) -> Option<(usize, usize, usize)> {
        let mut numbers = Vec::new();

        // Check if the input starts with "localnode"
        if input.starts_with("localnode") {
            numbers.push(0); // Add 0 as the node index
        }

        // Iterate through the string and extract numbers inside brackets
        let mut current_number = String::new();
        for c in input.chars() {
            if c.is_digit(10) {
                current_number.push(c);
            } else if c == ']' {
                if let Ok(num) = current_number.parse::<usize>() {
                    numbers.push(num);
                }
                current_number.clear();
            }
        }

        // Ensure we have exactly 3 numbers (node_idx, slot_idx, channel_idx)
        if numbers.len() == 3 {
            Some((numbers[0], numbers[1], numbers[2]))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct SweepModelAttributes {
    device_names: Vec<String>,
    step_names: Vec<String>,
    sweep_names: Vec<String>,
    bias_names: Vec<String>,
}

impl SweepModelAttributes {
    pub fn new() -> Self {
        SweepModelAttributes {
            device_names: Vec::new(),
            step_names: Vec::new(),
            sweep_names: Vec::new(),
            bias_names: Vec::new(),
        }
    }
}
