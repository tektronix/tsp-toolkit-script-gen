use std::{any::Any, collections::HashMap};

use crate::model::{chan_data::channel_range::ChannelRange, sweep_data::sweep_config::SweepConfig};

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

    fn define_bias_channels(&mut self, sweep_config: &SweepConfig) {
        let mut index = 1;
        for bias_channel in sweep_config.bias_channels.iter() {
            let instr_name = format!("bias{}", index);
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
                instr_name.clone() + ":SFUNCTION",
                bias_channel
                    .common_chan_attributes
                    .source_function
                    .value
                    .to_lowercase()
                    .clone(),
            );
            self.val_replacement_map.insert(
                instr_name.clone() + ":SRANGE",
                self.format_range(bias_channel.common_chan_attributes.source_range.clone()),
            );

            self.val_replacement_map.insert(
                instr_name.clone() + ":MFUNCTION",
                bias_channel
                    .common_chan_attributes
                    .meas_function
                    .value
                    .to_lowercase()
                    .clone(),
            );

            //sense mode exists only for SMU
            if let Some(sense_mode) = &bias_channel.common_chan_attributes.sense_mode {
                let sense_mode_key = format!("sense={}", sense_mode.value);
                if let Some(sense_mode_value) = bias_channel
                    .common_chan_attributes
                    .get_name_for(&sense_mode_key)
                {
                    self.val_replacement_map.insert(
                        instr_name.clone() + ":SENSE",
                        String::from(sense_mode_value),
                    );
                } else {
                    //TODO: error handling for sense mode value not found
                }
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
    /// the original value is returned. Otherwise, the scaled value is formatted using the `format` method.
    fn format_range(&self, range: ChannelRange) -> String {
        let mut result = String::from("NaN");
        if range.is_range_auto() || range.is_range_follow_limiti() {
            result = range.value;
        } else {
            let range_value = range.get_scaled_value();
            if let Some(value) = range_value {
                result = self.format(value);
            }
        }
        return result;
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
