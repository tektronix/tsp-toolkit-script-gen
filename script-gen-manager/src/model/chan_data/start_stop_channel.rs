use core::num;

use serde::{Deserialize, Serialize};

use super::default_channel::CommonChanAttributes;
use crate::{
    device::Device,
    instr_metadata::base_metadata::BaseMetadata,
    model::sweep_data::parameters::{ParameterFloat, ParameterString},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartStopChannel {
    pub common_chan_attributes: CommonChanAttributes,
    pub start: ParameterFloat,
    pub stop: ParameterFloat,
    pub style: ParameterString,
    pub list: Vec<ParameterFloat>,
    #[serde(default)]
    pub asymptote: f64,
}

impl StartStopChannel {
    pub fn new(chan_name: String, device: Device) -> Self {
        StartStopChannel {
            common_chan_attributes: CommonChanAttributes::new(chan_name, device),
            start: ParameterFloat::new("start", 0.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            stop: ParameterFloat::new("stop", 1.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            style: ParameterString::new("style"),
            list: Vec::new(),
            asymptote: 0.0,
        }
    }

    pub fn set_defaults(&mut self, steps_or_points: i32) {
        self.common_chan_attributes.set_defaults();
        self.start.value = 0.0;
        self.stop.value = 1.0;

        self.style.range = vec![
            BaseMetadata::STYLE_LIN.to_string(),
            BaseMetadata::STYLE_LOG.to_string(),
        ];
        self.style.value = BaseMetadata::STYLE_LIN.to_string();
        self.common_chan_attributes
            .update_region_constraints(self.start.value, self.stop.value);
        self.set_list(steps_or_points);
    }

    fn set_list(&mut self, steps_or_points: i32) {
        let num_points = steps_or_points;
        self.list = Vec::with_capacity(num_points.try_into().unwrap());

        for i in 0..num_points {
            let pf = ParameterFloat::new(
                &format!("list_{}", i),
                0.0,
                Some(BaseMetadata::UNIT_VOLTS.to_string()),
            );
            self.list.push(pf);
        }
    }

    pub fn evaluate(&mut self, list_size: usize) {
        self.common_chan_attributes.evaluate();
        self.determine_start_value();
        self.determine_stop_value();
        self.common_chan_attributes
            .update_region_constraints(self.start.value, self.stop.value);

        //List evaluation

        self.update_list(list_size);
    }

    fn update_list(&mut self, list_size: usize) {
        //get the right unit for the list value
        let unit = if self.common_chan_attributes.source_range.unit == BaseMetadata::UNIT_AMPERES {
            Some(BaseMetadata::UNIT_AMPERES.to_string())
        } else {
            Some(BaseMetadata::UNIT_VOLTS.to_string())
        };

        //The case where the list size is more than the requested size
        if self.list.len() > list_size {
            let pf = ParameterFloat::new(&format!("list_{}", 0), 0.0, unit.clone()); //Unused in this case
            self.list.resize(list_size, pf); //Simply truncate the list
        }
        if self.list.len() < list_size {
            //The case where the list size is less than the requested size
            let num_points = list_size - self.list.len();
            for i in 0..num_points {
                let pf = ParameterFloat::new(
                    &format!("list_{}", self.list.len() + i),
                    0.0,
                    unit.clone(),
                );
                self.list.push(pf); //Fill the list with default values
            }
        }

        for (_, pf) in self.list.iter_mut().enumerate() {
            if let Some(start_unit) = &pf.unit {
                if start_unit == &self.common_chan_attributes.source_range.unit {
                    pf.value = self.common_chan_attributes.source_range.limit(pf.value);
                } else {
                    //default to 0.0 if source function has been changed
                    pf.value = self.common_chan_attributes.source_range.limit(0.0);
                    pf.unit = Some(self.common_chan_attributes.source_range.unit.clone());
                }
            }
        }
    }

    fn determine_start_value(&mut self) {
        if let Some(start_unit) = &self.start.unit {
            if start_unit == &self.common_chan_attributes.source_range.unit {
                self.start.value = self
                    .common_chan_attributes
                    .source_range
                    .limit(self.start.value);
            } else {
                //default to 0.0 if source function has been changed
                self.start.value = self.common_chan_attributes.source_range.limit(0.0);
                self.start.unit = Some(self.common_chan_attributes.source_range.unit.clone());
            }
        } else {
            //TODO: handle error condition
            println!("start.unit is None");
        }

        if self.style.value == BaseMetadata::STYLE_LOG.to_string() {
            // start and stop must be on the same side of asymptote (0)
            if self.start.value >= BaseMetadata::MIN_LOG_VALUE {
                if self.stop.value < 0.0 {
                    // "flip" stop across asymptote (0.0)
                    self.stop.value = -self.stop.value;
                }
            } else if self.start.value <= -BaseMetadata::MIN_LOG_VALUE {
                if self.stop.value > 0.0 {
                    // "flip" stop across asymptote (0.0)
                    self.stop.value = -self.stop.value;
                }
            } else {
                // start == asymptote (0.0)
                // move start toward stop a little to get it off asymptote (0.0)
                if self.stop.value > 0.0 {
                    self.start.value = BaseMetadata::MIN_LOG_VALUE
                } else {
                    self.start.value = -BaseMetadata::MIN_LOG_VALUE
                }
            }
        }
    }

    fn determine_stop_value(&mut self) {
        if let Some(stop_unit) = &self.stop.unit {
            if stop_unit == &self.common_chan_attributes.source_range.unit {
                self.stop.value = self
                    .common_chan_attributes
                    .source_range
                    .limit(self.stop.value);
            } else {
                //default to 1.0 if source function has been changed
                self.stop.value = self.common_chan_attributes.source_range.limit(1.0);
                self.stop.unit = Some(self.common_chan_attributes.source_range.unit.clone());
            }
        } else {
            //TODO: handle error condition
            println!("bias.unit is None");
        }

        if self.style.value == BaseMetadata::STYLE_LOG.to_string() {
            // Start and stop must be on the same side of asymptote (0)
            if self.stop.value >= BaseMetadata::MIN_LOG_VALUE {
                if self.start.value < 0.0 {
                    // "flip" start across asymptote (0.0)
                    self.start.value = -self.start.value;
                }
            } else if self.stop.value <= -BaseMetadata::MIN_LOG_VALUE {
                if self.start.value > 0.0 {
                    // "flip" start across asymptote (0.0)
                    self.start.value = -self.start.value;
                }
            } else {
                // stop == asymptote (0.0)
                // Move stop toward start a little to get it off asymptote (0.0)
                if self.start.value > 0.0 {
                    self.stop.value = BaseMetadata::MIN_LOG_VALUE;
                } else {
                    self.stop.value = -BaseMetadata::MIN_LOG_VALUE;
                }
            }
        }
    }
}
