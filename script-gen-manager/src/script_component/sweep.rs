use std::any::Any;

use crate::{
    instr_metadata::base_metadata::BaseMetadata,
    model::{
        chan_data::{
            channel_range::ChannelRange, default_channel::CommonChanAttributes,
            start_stop_channel::StartStopChannel,
        },
        sweep_data::{
            parameters::{ParameterFloat, ParameterString},
            sweep_config::SweepConfig,
        },
    },
};

use super::function::FunctionModel;
use script_aggregator::script_buffer::ScriptBuffer;
use script_templates::{error::Result, FunctionTemplate};
use serde::Serialize;

/// [`SweepModel`] is an aggregation of [`FunctionModel`] that represents the `_Sweep()` function of the script.
/// This function executes a nested step/sweep operation and is optional in the generated script.
#[derive(Debug)]
pub struct SweepModel {
    type_: String,
    description: String,

    template: FunctionTemplate,
}

/// The values referenced by the Sweep template.
#[derive(Debug, Serialize)]
struct SweepContext {
    step_count: String,
    step_to_sweep_delay: String,
    sweep_points: String,
    nplc: String,
    aperture: String,
    measure_count: String,
    measure_delay: String,
    source_delay: String,

    bias_devices: Vec<ChannelContext>,
    step_devices: Vec<ChannelContext>,
    sweep_devices: Vec<ChannelContext>,
}

/// The values referenced by the Sweep template for a single bias, step or sweep channel.
#[derive(Debug, Serialize)]
struct ChannelContext {
    /// The name of the channel in the script (e.g., bias1, step1, sweep1)
    name: String,
    node: String,
    node_idx: Option<usize>,
    slot_idx: Option<usize>,
    channel_idx: Option<usize>,
    model: String,
    model_type: String,
    sfunction: String,
    srange: String,
    mfunction: String,
    mrange: String,
    sense: String,
    limiti: String,
    limitv: String,

    /// Only present for bias channels
    #[serde(skip_serializing_if = "Option::is_none")]
    bias: Option<String>,

    /// Only present for step and sweep channels
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    start_stop: Option<StartStopContext>,
}

/// The values referenced by the Sweep template for a step or sweep channel.
#[derive(Debug, Serialize)]
struct StartStopContext {
    mode: String,
    start: String,
    stop: String,
    list: String,
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

    fn get_template(&self) -> &FunctionTemplate {
        &self.template
    }

    fn to_script(
        &mut self,
        sweep_config: &SweepConfig,
        script_buffer: &mut ScriptBuffer,
    ) -> Result<()> {
        if sweep_config.step_channels.is_empty() && sweep_config.sweep_channels.is_empty() {
            script_buffer.postamble_append(&String::from(
                "-- no sweep ... requires at least 1 step channel or 1 sweep channel",
            ));
            Ok(())
        } else {
            let context = self.define_context(sweep_config);
            self.build(script_buffer, &context)
        }
    }
}

impl SweepModel {
    const DESCRIPTION: &'static str = "Configures a sweeping test.";
    // const NEGATIVE_NUMBER_NEAR_ZERO: f64 = -1.0e-30;

    pub fn new(template: FunctionTemplate) -> Self {
        Self {
            type_: template.type_().to_string(),
            description: Self::DESCRIPTION.to_string(),
            template,
        }
    }

    fn define_context(&self, sweep_config: &SweepConfig) -> SweepContext {
        let step_global = &sweep_config.step_global_parameters;
        let sweep_global = &sweep_config.sweep_global_parameters;

        let bias_devices = (1..)
            .zip(sweep_config.bias_channels.iter())
            .map(|(index, bias_channel)| ChannelContext {
                bias: Some(self.format(bias_channel.bias.value)),
                ..self.define_channel(format!("bias{index}"), &bias_channel.common_chan_attributes)
            })
            .collect();

        let step_devices: Vec<ChannelContext> = (1..)
            .zip(sweep_config.step_channels.iter())
            .map(|(index, step_channel)| {
                self.define_start_stop_channel(
                    format!("step{index}"),
                    &step_channel.start_stop_channel,
                    step_global.list_step,
                    step_global.step_points.value as usize,
                )
            })
            .collect();

        let sweep_devices = (1..)
            .zip(sweep_config.sweep_channels.iter())
            .map(|(index, sweep_channel)| {
                self.define_start_stop_channel(
                    format!("sweep{index}"),
                    &sweep_channel.start_stop_channel,
                    sweep_global.list_sweep,
                    sweep_global.sweep_points.value as usize,
                )
            })
            .collect();

        let (step_count, step_to_sweep_delay) = if step_devices.is_empty() {
            (String::from("1"), String::from("0"))
        } else {
            (
                step_global.step_points.value.to_string(),
                self.format(step_global.step_to_sweep_delay.value),
            )
        };

        let timing = &sweep_config.global_parameters.sweep_timing_config;
        let smu_timing = &timing.smu_timing;
        let (nplc, aperture) = if smu_timing.nplc_type.value == "NPLC" {
            (self.format(smu_timing.nplc.value), String::from("nil"))
        } else {
            (String::from("nil"), self.format(smu_timing.aperture.value))
        };

        let measure_delay = if smu_timing.measure_auto_delay.value == BaseMetadata::OFF_VALUE {
            self.format(smu_timing.measure_delay.value)
        } else {
            String::from("nil")
        };

        let source_delay = if smu_timing.source_auto_delay.value == BaseMetadata::OFF_VALUE {
            self.format(smu_timing.source_delay.value)
        } else {
            String::from("nil")
        };

        SweepContext {
            step_count,
            step_to_sweep_delay,
            sweep_points: sweep_global.sweep_points.value.to_string(),
            nplc,
            aperture,
            measure_count: self.format(timing.measure_count.value.into()),
            measure_delay,
            source_delay,

            bias_devices,
            step_devices,
            sweep_devices,
        }
    }

    /// Builds the template values common to bias, step and sweep channels.
    fn define_channel(&self, name: String, attributes: &CommonChanAttributes) -> ChannelContext {
        let device = &attributes.device;
        let (node_idx, slot_idx, channel_idx) = Self::extract_indices(&device.get_id())
            .map_or((None, None, None), |(node, slot, channel)| {
                (Some(node), Some(slot), Some(channel))
            });

        //sense mode exists only for SMU
        let sense = attributes.sense_mode.as_ref().map_or_else(
            || BaseMetadata::UNDEFINED.to_string(),
            |sense_mode| {
                attributes
                    .get_name_for(&format!("sense={}", sense_mode.value))
                    .unwrap_or_default()
                    .to_owned()
            },
        );

        //source limits exist only for SMU
        let format_limit = |limit: &Option<ParameterFloat>| {
            limit
                .as_ref()
                .map_or_else(|| String::from("nil"), |limit| self.format(limit.value))
        };

        ChannelContext {
            name,
            node: device.get_node_id(),
            node_idx,
            slot_idx,
            channel_idx,
            model: device.get_model(),
            model_type: device.device_type.to_string(),
            sfunction: Self::get_function_value(&attributes.source_function),
            srange: Self::get_range_value(&attributes.source_range),
            mfunction: Self::get_function_value(&attributes.meas_function),
            mrange: Self::get_range_value(&attributes.meas_range),
            sense,
            limiti: format_limit(&attributes.source_limiti),
            limitv: format_limit(&attributes.source_limitv),
            bias: None,
            start_stop: None,
        }
    }

    /// Builds the template values for a step or sweep channel.
    fn define_start_stop_channel(
        &self,
        name: String,
        channel: &StartStopChannel,
        is_list: bool,
        list_len: usize,
    ) -> ChannelContext {
        ChannelContext {
            start_stop: Some(StartStopContext {
                mode: if is_list {
                    "LIST".to_string()
                } else {
                    channel.style.value.clone()
                },
                start: self.format(channel.start.value),
                stop: self.format(channel.stop.value),
                list: Self::process_list(is_list, &channel.list, list_len),
            }),
            ..self.define_channel(name, &channel.common_chan_attributes)
        }
    }

    fn get_range_value(channel_range: &ChannelRange) -> String {
        if channel_range.is_range_auto() {
            "CONSTANTS.AUTO".to_string()
        } else {
            Self::format_range(channel_range.clone())
        }
    }

    //Returns the value used in the script
    fn get_function_value(source_function: &ParameterString) -> String {
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

    fn process_list(
        is_list: bool,
        list: &[ParameterFloat],
        len: usize, // Default length for list values
    ) -> String {
        //Default value for list is nil
        if is_list {
            let mut new_list = list
                .iter()
                .map(|item| item.value.to_string())
                .collect::<Vec<_>>();
            if !new_list.is_empty() && new_list.len() > len {
                new_list.truncate(len);
            }

            //Fill in list values
            format!("{{ {} }}", new_list.join(", "))
        } else {
            "nil".to_string()
        }
    }

    /// Formats the value of a `ChannelRange` object into a string.
    ///
    /// # Arguments
    /// * `range` - A `ChannelRange` object containing the range value.
    ///
    /// # Returns
    /// A formatted string representing the range value. If the range is set to auto or follow limit,
    /// the original value is returned. Otherwise, the scaled value is formatted with 3 decimal places.
    fn format_range(range: ChannelRange) -> String {
        let result = if range.is_range_auto() || range.is_range_follow_limiti() {
            range.value
        } else {
            let range_value = range.get_scaled_value();
            range_value.map_or_else(
                || "NaN".to_string(),
                |value| {
                    if value.abs() < 1e-3 || value.abs() >= 1e3 {
                        format!("{value:.3e}")
                    } else {
                        format!("{value:.3}")
                    }
                },
            )
        };

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
    fn extract_indices(input: &str) -> Option<(usize, usize, usize)> {
        let mut numbers = Vec::new();

        // Check if the input starts with "localnode"
        if input.starts_with("localnode") {
            numbers.push(0); // Add 0 as the node index
        }

        // Iterate through the string and extract numbers inside brackets
        let mut current_number = String::new();
        for c in input.chars() {
            if c.is_ascii_digit() {
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
