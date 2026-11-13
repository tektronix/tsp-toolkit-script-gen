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
    #[must_use]
    pub fn new(chan_name: String, device: Device) -> Self {
        Self {
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
        self.set_list(steps_or_points);
    }

    /// Retains compatible sweep values while applying the new device's limits and units.
    pub fn migrate_to_device(&mut self, device: Device) {
        self.common_chan_attributes.migrate_to_device(device);

        let unit = self.common_chan_attributes.source_range.unit.clone();
        // Voltage and current values are not interchangeable; keep values only for matching units.
        let start_value = if self.start.unit.as_deref() == Some(unit.as_str()) {
            self.start.value
        } else {
            0.0
        };
        self.start.value = self.common_chan_attributes.source_range.limit(start_value);
        self.start.unit = Some(unit.clone());

        let stop_value = if self.stop.unit.as_deref() == Some(unit.as_str()) {
            self.stop.value
        } else {
            1.0
        };
        self.stop.value = self.common_chan_attributes.source_range.limit(stop_value);
        self.stop.unit = Some(unit.clone());

        for point in &mut self.list {
            let value = if point.unit.as_deref() == Some(unit.as_str()) {
                point.value
            } else {
                0.0
            };
            point.value = self.common_chan_attributes.source_range.limit(value);
            point.unit = Some(unit.clone());
        }

        self.style.range = vec![
            BaseMetadata::STYLE_LIN.to_string(),
            BaseMetadata::STYLE_LOG.to_string(),
        ];
        if !self.style.range.contains(&self.style.value) {
            self.style.value = BaseMetadata::STYLE_LIN.to_string();
        }
    }

    fn set_list(&mut self, steps_or_points: i32) {
        let num_points = steps_or_points;
        self.list = Vec::with_capacity(num_points.try_into().unwrap());

        for i in 0..num_points {
            let pf = ParameterFloat::new(
                &format!("list_{i}"),
                0.0,
                Some(BaseMetadata::UNIT_VOLTS.to_string()),
            );
            self.list.push(pf);
        }
    }

    /// # Panics
    /// May panic if any of the values in this list are not comparable. This is an unexpected situation
    pub fn evaluate(&mut self, list_size: usize, is_list_enabled: bool) {
        self.common_chan_attributes.evaluate();
        self.determine_start_value();
        self.determine_stop_value();

        //List evaluation

        self.update_list(list_size);

        if is_list_enabled {
            let min_value = self
                .list
                .iter()
                .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
                .unwrap();
            let max_value = self
                .list
                .iter()
                .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
                .unwrap();

            self.common_chan_attributes
                .evaluate_source_limits(min_value, max_value);
        } else {
            self.common_chan_attributes
                .evaluate_source_limits(&self.start, &self.stop);
        }
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

        for pf in &mut self.list {
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

        if self.style.value == BaseMetadata::STYLE_LOG {
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
                    self.start.value = BaseMetadata::MIN_LOG_VALUE;
                } else {
                    self.start.value = -BaseMetadata::MIN_LOG_VALUE;
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

        if self.style.value == BaseMetadata::STYLE_LOG {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::system_info::Slot;

    fn device(module: &str, channel: i32) -> Device {
        let slot = Slot {
            slot_id: "slot[1]".to_string(),
            module: module.to_string(),
        };
        Device::new("localnode", "MP5103", &slot, channel)
    }

    #[test]
    fn migration_preserves_values_and_clamps_out_of_range_points() {
        let old_device = device("MSMU60-2", 1);
        let new_device = device("MPSU50-2ST", 2);
        let mut channel = StartStopChannel::new("sweep1".to_string(), old_device);
        channel.set_defaults(3);
        channel.start.value = 1.0;
        channel.stop.value = 200.0;
        channel.list = vec![
            ParameterFloat::new("list_0", 10.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            ParameterFloat::new("list_1", 40.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
            ParameterFloat::new("list_2", 200.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
        ];

        channel.migrate_to_device(new_device);

        assert_eq!(channel.list.len(), 3);
        assert_eq!(
            channel
                .list
                .iter()
                .map(|point| point.value)
                .collect::<Vec<_>>(),
            vec![10.0, 40.0, 50.1]
        );
        assert!((channel.stop.value - 50.1).abs() < 1e-9);
    }

    #[test]
    fn migration_defaults_values_with_units_unsupported_by_target_function() {
        let old_device = device("MSMU60-2", 1);
        let new_device = device("MPSU50-2ST", 2);
        let mut channel = StartStopChannel::new("sweep1".to_string(), old_device);
        channel.set_defaults(3);
        channel.common_chan_attributes.source_function.value =
            BaseMetadata::FUNCTION_CURRENT.to_string();
        channel.common_chan_attributes.source_range.value = "100 mA".to_string();
        channel.start =
            ParameterFloat::new("start", 0.1, Some(BaseMetadata::UNIT_AMPERES.to_string()));
        channel.stop =
            ParameterFloat::new("stop", 0.5, Some(BaseMetadata::UNIT_AMPERES.to_string()));
        channel.list = vec![
            ParameterFloat::new("list_0", 0.1, Some(BaseMetadata::UNIT_AMPERES.to_string())),
            ParameterFloat::new("list_1", 0.3, Some(BaseMetadata::UNIT_AMPERES.to_string())),
            ParameterFloat::new("list_2", 0.5, Some(BaseMetadata::UNIT_AMPERES.to_string())),
        ];

        channel.migrate_to_device(new_device);

        assert_eq!(
            channel.common_chan_attributes.source_function.value,
            BaseMetadata::FUNCTION_VOLTAGE
        );
        assert_eq!(channel.start.value, 0.0);
        assert_eq!(channel.stop.value, 1.0);
        assert_eq!(channel.list.len(), 3);
        assert!(channel.list.iter().all(|point| {
            point.value == 0.0 && point.unit.as_deref() == Some(BaseMetadata::UNIT_VOLTS)
        }));
    }
}
