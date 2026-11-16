use std::any::Any;

use crate::model::sweep_data::sweep_config::SweepConfig;

use super::function::FunctionModel;
use script_aggregator::script_buffer::ScriptBuffer;
use script_templates::{error::Result, FunctionTemplate};
use serde::Serialize;

/// [`DataReportModel`] is an aggregation of [`FunctionModel`] that represents the `_DataReport()` function of the script.
/// This is an optional function in the generated script.
#[derive(Debug)]
pub struct DataReportModel {
    type_: String,
    select_mode: SelectMode,
    description: String,
    template: FunctionTemplate,
}

/// The values referenced by the DataReport template.
#[derive(Debug, Serialize)]
struct DataReportContext {
    reading_buffers: &'static str,
    reading_buffer_names: &'static str,
    reading_buffer_smu_names: &'static str,
    wait_interval: u32,
    max_readings_to_return: u32,

    tag_data_report: &'static str,
    tag_sweep_start: &'static str,
    tag_start: &'static str,
    tag_name: &'static str,
    tag_expected_count: &'static str,
    tag_pts_in_buff: &'static str,
    tag_pts_returned: &'static str,
    tag_base_time_stamp: &'static str,
    tag_readings: &'static str,
    tag_timestamps: &'static str,
    tag_srcvals: &'static str,
    tag_end: &'static str,
    tag_complete: &'static str,
}

impl FunctionModel for DataReportModel {
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
        let context = DataReportContext {
            reading_buffers: "{}",
            reading_buffer_names: "{}",
            reading_buffer_smu_names: "{}",
            wait_interval: 1,
            max_readings_to_return: 100,

            tag_data_report: "Data",
            tag_sweep_start: "SWEEP-START",
            tag_start: "START",
            tag_name: "NAME",
            tag_expected_count: "EXPECTED-COUNT",
            tag_pts_in_buff: "PTS-IN-BUFF",
            tag_pts_returned: "PTS-RETURNED",
            tag_base_time_stamp: "BASE-TIME-STAMP",
            tag_readings: "READINGS",
            tag_timestamps: "TIMESTAMPS",
            tag_srcvals: "SRCVALS",
            tag_end: "END",
            tag_complete: "COMPLETE",
        };

        self.build(script_buffer, &context)
    }
}

impl DataReportModel {
    const DESCRIPTION: &'static str = "This script returns a series of reading buffers. ";

    pub fn new(template: FunctionTemplate) -> Self {
        Self {
            type_: template.type_().to_string(),
            select_mode: SelectMode::Auto,
            description: Self::DESCRIPTION.to_string(),
            template,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum SelectMode {
    Auto,
    Custom,
}
