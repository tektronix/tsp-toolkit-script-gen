use std::{any::Any, collections::HashMap};

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

/// [`SweepModel`] is an aggregation of [`FunctionModel`] that represents the `Sweep()` function of the script.
/// This function executes a nested step/sweep operation and is optional in the generated script.
#[derive(Debug)]
pub struct SweepModel {
    type_: String,
    description: String,

    template: FunctionTemplate,
}

/// The values referenced by the Sweep template.
///
/// Everything that is known when the script is generated (channel locations, event IDs, loop
/// counts, which trigger-model blocks are needed, etc.) is resolved here so that the template
/// can emit straight-line TSP code for each channel.
#[derive(Debug, Serialize)]
struct SweepContext {
    /// Messages for configuration problems that must stop the script before it runs.
    errors: Vec<String>,

    /// The number of points of the step channels, if there are any.
    step_points: Option<String>,
    /// The number of points of the sweep channels, if there are any.
    sweep_points: Option<String>,
    /// Only present if there are both step and sweep channels.
    step_to_sweep_delay: Option<String>,
    nplc: Option<String>,
    aperture: Option<String>,
    source_delay: Option<String>,
    measure_delay: Option<String>,
    measure_count: String,

    /// The number of iterations of the step loop, if there are step channels.
    step_count: Option<String>,
    /// The number of iterations of the sweep loop, if there are sweep channels.
    sweep_count: Option<String>,
    /// The total number of points, used by the bias channels.
    total_count: String,

    /// The nodes used by the channels (e.g., localnode, node[2])
    nodes: Vec<String>,

    /// The bias, step and sweep channels, in that order.
    channels: Vec<ChannelContext>,

    /// The source/measure events of the first channel of each role, which the other roles wait on.
    bias_source: Option<String>,
    bias_measure: Option<String>,
    step_source: Option<String>,
    step_measure: Option<String>,
    sweep_source: Option<String>,
    sweep_measure: Option<String>,

    /// The header line of the results table.
    results_header: String,
    /// The buffers that make up the columns of the results table.
    results: Vec<ResultContext>,
}

/// A column of the results table.
#[derive(Debug, Serialize)]
struct ResultContext {
    /// The name of the channel in the script (e.g., bias1)
    channel: String,
    /// e.g., defbuffer1
    buffer: &'static str,
}

/// The values referenced by the Sweep template for a single bias, step or sweep channel.
#[derive(Debug, Serialize)]
struct ChannelContext {
    /// "bias", "step" or "sweep"
    role: &'static str,
    /// The 1-based index of the channel within its role.
    role_index: usize,
    /// The name of the channel in the script (e.g., bias1, step1, sweep1)
    name: String,
    /// e.g., localnode, node[2]
    node: String,
    /// "Bias", "Step" or "Sweep"
    role_title: &'static str,
    slot_idx: usize,
    channel_idx: usize,
    /// The channel table of the slot ("smu" or "psu")
    channel_type: &'static str,
    /// Whether this is the first channel to use its slot, which is responsible for checking the
    /// installed module and aborting the slot's trigger models.
    first_in_slot: bool,
    model: String,
    is_smu: bool,

    /// The names of the script variables holding this channel's notify events.
    source_event: String,
    measure_event: String,
    /// The numbers of this channel's notify events (`EVENT_NOTIFY<n>` of its slot), if available.
    source_event_id: Option<usize>,
    measure_event_id: Option<usize>,

    sense: Option<String>,

    /// Only set for SMUs.
    source_func: Option<String>,
    /// The suffix of the source attributes and functions for the source function ("v" or "i").
    source_unit: Option<&'static str>,
    source_autorange: bool,
    source_range: Option<String>,
    limiti: Option<String>,
    limitv: Option<String>,

    /// Exactly one of these is present for a valid channel.
    level: Option<String>,
    list: Option<String>,
    sweep: Option<SweepValuesContext>,

    /// The suffix of `trigger.measure.*()` ("i", "v" or "iv")
    measure_func: &'static str,
    /// The suffix of the measure range attributes ("v" or "i"), absent for combined I/V.
    measure_unit: Option<&'static str>,
    measure_autorange: bool,
    measure_range: Option<String>,

    /// Waits for the other channels of the same role to finish sourcing (first channel only).
    source_waits: Vec<WaitContext>,
    /// Waits for the other channels of the same role to finish measuring (first channel only).
    measure_waits: Vec<WaitContext>,
    /// Waits for the channels of the other roles to finish measuring.
    dependency_waits: Vec<WaitContext>,
}

/// A linear or log source sweep.
#[derive(Debug, Serialize)]
struct SweepValuesContext {
    /// "linear" or "log"
    kind: &'static str,
    start: String,
    stop: String,
    count: String,
}

/// A trigger-model wait block for one or more events.
#[derive(Debug, Serialize)]
struct WaitContext {
    /// The name of the block, without the channel's block-name prefix.
    block: String,
    first: String,
    rest: Vec<String>,
}

/// The role a channel plays in the sweep.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Bias,
    Step,
    Sweep,
}

impl Role {
    const fn name(self) -> &'static str {
        match self {
            Self::Bias => "bias",
            Self::Step => "step",
            Self::Sweep => "sweep",
        }
    }

    const fn title(self) -> &'static str {
        match self {
            Self::Bias => "Bias",
            Self::Step => "Step",
            Self::Sweep => "Sweep",
        }
    }
}

/// How a step or sweep channel sources its values.
struct StartStopValues<'a> {
    channel: &'a StartStopChannel,
    is_list: bool,
    points: usize,
    /// The script variable holding the number of points.
    points_name: &'static str,
}

/// Assigns the trigger-model notify events of each slot, in order of use.
#[derive(Default)]
struct EventAllocator {
    used: HashMap<String, usize>,
}

impl EventAllocator {
    /// Returns the number of the next notify event of `slot`, or `None` if all `available`
    /// events are in use.
    fn next(&mut self, slot: &str, available: usize) -> Option<usize> {
        let used = self.used.entry(slot.to_string()).or_default();
        if *used < available {
            *used += 1;
            Some(*used)
        } else {
            None
        }
    }
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

    /// The number of trigger-model notify events available in a slot containing an SMU.
    const SMU_EVENTS: usize = 8;
    /// The number of trigger-model notify events available in a slot containing a PSU.
    const PSU_EVENTS: usize = 16;
    /// The maximum number of events a single trigger-model wait block can wait on.
    const MAX_WAIT_EVENTS: usize = 3;

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

        let mut errors = Vec::new();
        let mut events = EventAllocator::default();

        // Channels are defined in the order biases, steps, sweeps so that notify events are
        // assigned in that order.
        let mut biases = Vec::new();
        for (index, bias_channel) in (1..).zip(&sweep_config.bias_channels) {
            let mut channel = self.define_channel(
                Role::Bias,
                index,
                &bias_channel.common_chan_attributes,
                &mut events,
                &mut errors,
            );
            channel.level = Some(self.format(bias_channel.bias.value));
            biases.push(channel);
        }

        let mut steps = Vec::new();
        let mut step_count = None;
        for (index, step_channel) in (1..).zip(&sweep_config.step_channels) {
            let values = StartStopValues {
                channel: &step_channel.start_stop_channel,
                is_list: step_global.list_step,
                points: step_global.step_points.value as usize,
                points_name: "numberOfSteps",
            };
            let (channel, count) = self.define_start_stop_channel(
                Role::Step,
                index,
                &values,
                &mut events,
                &mut errors,
            );
            step_count.get_or_insert(count);
            steps.push(channel);
        }

        let mut sweeps = Vec::new();
        let mut sweep_count = None;
        for (index, sweep_channel) in (1..).zip(&sweep_config.sweep_channels) {
            let values = StartStopValues {
                channel: &sweep_channel.start_stop_channel,
                is_list: sweep_global.list_sweep,
                points: sweep_global.sweep_points.value as usize,
                points_name: "numberOfSweeps",
            };
            let (channel, count) = self.define_start_stop_channel(
                Role::Sweep,
                index,
                &values,
                &mut events,
                &mut errors,
            );
            sweep_count.get_or_insert(count);
            sweeps.push(channel);
        }

        let first_events = |channels: &[ChannelContext]| {
            channels.first().map_or((None, None), |c| {
                (Some(c.source_event.clone()), Some(c.measure_event.clone()))
            })
        };
        let (bias_source, bias_measure) = first_events(&biases);
        let (step_source, step_measure) = first_events(&steps);
        let (sweep_source, sweep_measure) = first_events(&sweeps);

        Self::define_role_waits(&mut biases, "_waitall_bias_", "_waitall_bias_meas_");
        Self::define_role_waits(&mut steps, "_step_waitall_", "_step_other_measure_waitall_");
        Self::define_role_waits(
            &mut sweeps,
            "_sweep_waitall_",
            "_sweep_other_measure_waitall_",
        );

        // Steps and sweeps wait for the measurements of the other roles before looping.
        let step_dependencies: Vec<String> = [&sweep_measure, &bias_measure]
            .into_iter()
            .flatten()
            .cloned()
            .collect();
        for step in &mut steps {
            step.dependency_waits =
                Self::wait_blocks("_step_wait_sweep_bias_meas", &step_dependencies);
        }
        let sweep_dependencies: Vec<String> = [&step_measure, &bias_measure]
            .into_iter()
            .flatten()
            .cloned()
            .collect();
        for sweep in &mut sweeps {
            sweep.dependency_waits = Self::wait_blocks("_sweep_meas done", &sweep_dependencies);
        }

        let total_count = match (&step_count, &sweep_count) {
            (Some(step), Some(sweep)) => format!("{step} * {sweep}"),
            (Some(count), None) | (None, Some(count)) => count.clone(),
            (None, None) => String::from("0"),
        };

        let timing = &sweep_config.global_parameters.sweep_timing_config;
        let smu_timing = &timing.smu_timing;
        let (nplc, aperture) = if smu_timing.nplc_type.value == "NPLC" {
            (Some(self.format(smu_timing.nplc.value)), None)
        } else {
            (None, Some(self.format(smu_timing.aperture.value)))
        };
        let measure_delay = (smu_timing.measure_auto_delay.value == BaseMetadata::OFF_VALUE)
            .then(|| self.format(smu_timing.measure_delay.value));
        let source_delay = (smu_timing.source_auto_delay.value == BaseMetadata::OFF_VALUE)
            .then(|| self.format(smu_timing.source_delay.value));

        let mut channels: Vec<ChannelContext> =
            biases.into_iter().chain(steps).chain(sweeps).collect();

        let mut nodes: Vec<String> = Vec::new();
        let mut slots: Vec<(String, usize)> = Vec::new();
        let mut results_header = String::from("Reading Number");
        let mut results = Vec::new();
        for channel in &mut channels {
            if !nodes.contains(&channel.node) {
                nodes.push(channel.node.clone());
            }
            let slot = (channel.node.clone(), channel.slot_idx);
            if !slots.contains(&slot) {
                slots.push(slot);
                channel.first_in_slot = true;
            }
            for &(heading, buffer) in Self::measure_buffers(channel.measure_func) {
                results_header.push_str(&format!(
                    ",{}[{}].{heading}",
                    channel.role, channel.role_index
                ));
                results.push(ResultContext {
                    channel: channel.name.clone(),
                    buffer,
                });
            }
        }

        SweepContext {
            errors,

            step_points: step_count
                .is_some()
                .then(|| step_global.step_points.value.to_string()),
            sweep_points: sweep_count
                .is_some()
                .then(|| sweep_global.sweep_points.value.to_string()),
            step_to_sweep_delay: (step_count.is_some() && sweep_count.is_some())
                .then(|| self.format(step_global.step_to_sweep_delay.value)),
            nplc,
            aperture,
            source_delay,
            measure_delay,
            measure_count: self.format(timing.measure_count.value.into()),

            step_count,
            sweep_count,
            total_count,

            nodes,
            channels,

            bias_source,
            bias_measure,
            step_source,
            step_measure,
            sweep_source,
            sweep_measure,

            results_header,
            results,
        }
    }

    /// Builds the template values common to bias, step and sweep channels.
    fn define_channel(
        &self,
        role: Role,
        index: usize,
        attributes: &CommonChanAttributes,
        events: &mut EventAllocator,
        errors: &mut Vec<String>,
    ) -> ChannelContext {
        let name = format!("{}{index}", role.name());
        let device = &attributes.device;
        let id = device.get_id();
        let (node_idx, slot_idx, channel_idx) = Self::extract_indices(&id).unwrap_or_else(|| {
            errors.push(format!(
                "Unable to determine the location of channel '{id}'"
            ));
            (0, 0, 0)
        });

        let node = if node_idx == 0 {
            String::from("localnode")
        } else {
            if node_idx > 63 {
                errors.push(format!(
                    "The given node index '{node_idx}' is not a valid node index"
                ));
            }
            format!("node[{node_idx}]")
        };
        let slot = format!("{node}.slot[{slot_idx}]");

        let model = device.get_model();
        let is_smu = if model.contains("MPSU") {
            false
        } else {
            if !model.contains("MSMU") {
                errors.push(format!("Unable to determine device model of '{id}'"));
            }
            true
        };

        let available_events = if is_smu {
            Self::SMU_EVENTS
        } else {
            Self::PSU_EVENTS
        };
        let mut next_event = || {
            events.next(&slot, available_events).or_else(|| {
                errors.push(format!(
                    "Unable to assign event IDs to {name}: all event IDs for {slot} have been consumed."
                ));
                None
            })
        };
        let source_event_id = next_event();
        let measure_event_id = next_event();

        //sense mode exists only for SMU
        let sense = attributes
            .sense_mode
            .as_ref()
            .filter(|_| is_smu)
            .and_then(|sense_mode| attributes.get_name_for(&format!("sense={}", sense_mode.value)))
            .filter(|sense| !sense.is_empty() && *sense != BaseMetadata::UNDEFINED)
            .map(str::to_string);

        let format_limit =
            |limit: &Option<ParameterFloat>| limit.as_ref().map(|limit| self.format(limit.value));

        // PSUs only source voltage and have no source function, range or voltage limit settings.
        let source_function = Self::get_function_value(&attributes.source_function);
        let source_unit = if is_smu {
            Self::unit(&source_function)
        } else {
            Some("v")
        };
        let (source_autorange, source_range) = if is_smu && source_unit.is_some() {
            Self::get_range_value(&attributes.source_range)
        } else {
            (false, None)
        };

        let measure_function = Self::get_function_value(&attributes.meas_function);
        let measure_unit = Self::unit(&measure_function);
        let (measure_autorange, measure_range) = if is_smu && measure_unit.is_some() {
            Self::get_range_value(&attributes.meas_range)
        } else {
            (false, None)
        };

        ChannelContext {
            source_event: format!("{name}_source_event"),
            measure_event: format!("{name}_measure_event"),
            source_event_id,
            measure_event_id,

            role: role.name(),
            role_index: index,
            role_title: role.title(),
            name,
            node,
            slot_idx,
            channel_idx,
            channel_type: if is_smu { "smu" } else { "psu" },
            first_in_slot: false,
            model,
            is_smu,

            sense,

            source_func: source_unit.filter(|_| is_smu).map(|_| source_function),
            source_unit,
            source_autorange,
            source_range,
            limiti: format_limit(&attributes.source_limiti),
            limitv: format_limit(&attributes.source_limitv).filter(|_| is_smu),

            level: None,
            list: None,
            sweep: None,

            measure_func: match measure_function.as_str() {
                "FUNC_DC_VOLTAGE" => "v",
                "FUNC_DC_CURRENT" => "i",
                _ => "iv",
            },
            measure_unit,
            measure_autorange,
            measure_range,

            source_waits: Vec::new(),
            measure_waits: Vec::new(),
            dependency_waits: Vec::new(),
        }
    }

    /// Builds the template values for a step or sweep channel.
    ///
    /// Returns the channel and the number of points it sources.
    fn define_start_stop_channel(
        &self,
        role: Role,
        index: usize,
        values: &StartStopValues,
        events: &mut EventAllocator,
        errors: &mut Vec<String>,
    ) -> (ChannelContext, String) {
        let channel = values.channel;
        let mut context =
            self.define_channel(role, index, &channel.common_chan_attributes, events, errors);

        let count = if values.is_list {
            let list: Vec<String> = channel
                .list
                .iter()
                .take(values.points)
                .map(|item| item.value.to_string())
                .collect();
            context.list = Some(format!("{{ {} }}", list.join(", ")));
            list.len().to_string()
        } else {
            let kind = match channel.style.value.as_str() {
                BaseMetadata::STYLE_LIN => Some("linear"),
                BaseMetadata::STYLE_LOG => Some("log"),
                style => {
                    errors.push(format!(
                        "Unknown source style '{style}' for {}",
                        context.name
                    ));
                    None
                }
            };
            context.sweep = kind.map(|kind| SweepValuesContext {
                kind,
                start: self.format(channel.start.value),
                stop: self.format(channel.stop.value),
                count: values.points_name.to_string(),
            });
            values.points_name.to_string()
        };

        (context, count)
    }

    /// Makes the first channel of a role wait for all of the other channels of that role to
    /// finish sourcing and measuring.
    fn define_role_waits(channels: &mut [ChannelContext], source_block: &str, measure_block: &str) {
        let Some((first, others)) = channels.split_first_mut() else {
            return;
        };
        let source_events: Vec<String> = others.iter().map(|c| c.source_event.clone()).collect();
        let measure_events: Vec<String> = others.iter().map(|c| c.measure_event.clone()).collect();
        first.source_waits = Self::wait_blocks(source_block, &source_events);
        first.measure_waits = Self::wait_blocks(measure_block, &measure_events);
    }

    /// Splits waiting on all of `events` into as many wait blocks as needed. The blocks are named
    /// `block` followed by their 1-based index.
    fn wait_blocks(block: &str, events: &[String]) -> Vec<WaitContext> {
        (1..)
            .zip(events.chunks(Self::MAX_WAIT_EVENTS))
            .map(|(index, chunk)| WaitContext {
                block: format!("{block}{index}"),
                first: chunk[0].clone(),
                rest: chunk[1..].to_vec(),
            })
            .collect()
    }

    /// The results-table headings and buffer names for the given measure function.
    fn measure_buffers(measure_func: &str) -> &'static [(&'static str, &'static str)] {
        match measure_func {
            "iv" => &[("CURRENT", "defbuffer1"), ("VOLTAGE", "defbuffer2")],
            "v" => &[("VOLTAGE", "defbuffer1")],
            _ => &[("CURRENT", "defbuffer1")],
        }
    }

    /// The suffix of the source/measure attributes for the given function, if it has one.
    fn unit(function: &str) -> Option<&'static str> {
        match function {
            "FUNC_DC_VOLTAGE" => Some("v"),
            "FUNC_DC_CURRENT" => Some("i"),
            _ => None,
        }
    }

    /// Returns whether the range is auto and, if not, the range value used in the script.
    fn get_range_value(channel_range: &ChannelRange) -> (bool, Option<String>) {
        if channel_range.is_range_auto() {
            (true, None)
        } else {
            (false, Some(Self::format_range(channel_range.clone())))
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

#[cfg(test)]
mod tests {
    use super::{EventAllocator, SweepModel};

    #[test]
    fn wait_blocks_split_events() {
        let events: Vec<String> = (1..=5).map(|i| format!("e{i}")).collect();
        let blocks = SweepModel::wait_blocks("wait_", &events);
        let blocks: Vec<(&str, &str, Vec<&str>)> = blocks
            .iter()
            .map(|b| {
                (
                    b.block.as_str(),
                    b.first.as_str(),
                    b.rest.iter().map(String::as_str).collect(),
                )
            })
            .collect();
        assert_eq!(
            blocks,
            [
                ("wait_1", "e1", vec!["e2", "e3"]),
                ("wait_2", "e4", vec!["e5"]),
            ]
        );
        assert!(SweepModel::wait_blocks("wait_", &[]).is_empty());
    }

    #[test]
    fn events_are_allocated_per_slot() {
        let mut events = EventAllocator::default();
        let slot1 = "localnode.slot[1]";
        let slot2 = "localnode.slot[2]";
        assert_eq!(events.next(slot1, 2), Some(1));
        assert_eq!(events.next(slot2, 2), Some(1));
        assert_eq!(events.next(slot1, 2), Some(2));
        assert_eq!(events.next(slot1, 2), None);
    }

    #[test]
    fn extract_indices_from_remote_node() {
        assert_eq!(
            SweepModel::extract_indices("node[37].slot[1].smu[1]"),
            Some((37, 1, 1))
        );
    }

    #[test]
    fn extract_indices_from_localnode() {
        assert_eq!(
            SweepModel::extract_indices("localnode.slot[2].psu[2]"),
            Some((0, 2, 2))
        );
    }

    #[test]
    fn extract_indices_requires_three_indices() {
        assert_eq!(SweepModel::extract_indices("localnode.slot[1]"), None);
        assert_eq!(
            SweepModel::extract_indices("node[1].slot[2].smu[1].x[4]"),
            None
        );
    }
}
