use serde::{Deserialize, Serialize};

use super::default_channel::CommonChanAttributes;
use crate::{
    device::Device, instr_metadata::base_metadata::BaseMetadata,
    model::sweep_data::parameters::ParameterFloat,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasChannel {
    pub common_chan_attributes: CommonChanAttributes,
    pub bias: ParameterFloat,
}

impl BiasChannel {
    #[must_use]
    pub fn new(chan_name: String, device: Device) -> Self {
        let mut bias_channel = Self {
            common_chan_attributes: CommonChanAttributes::new(chan_name, device),
            bias: ParameterFloat::new("bias", 0.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
        };

        // Call set_defaults on the common_chan_attributes field
        bias_channel.common_chan_attributes.set_defaults();
        bias_channel
    }

    pub fn migrate_to_device(&mut self, device: Device) {
        self.common_chan_attributes.migrate_to_device(device);

        let unit = self.common_chan_attributes.source_range.unit.clone();
        // A bias cannot be converted when the new source function changes its physical unit.
        let value = if self.bias.unit.as_deref() == Some(unit.as_str()) {
            self.bias.value
        } else {
            0.0
        };
        self.bias.value = self.common_chan_attributes.source_range.limit(value);
        self.bias.unit = Some(unit);
        self.common_chan_attributes
            .evaluate_source_limits(&self.bias, &self.bias);
    }

    pub fn evaluate(&mut self) {
        self.common_chan_attributes.evaluate();
        self.determine_bias_value();
        self.common_chan_attributes
            .evaluate_source_limits(&self.bias, &self.bias); //Use the bias value for both start and stop as the absolute max is used
    }

    fn determine_bias_value(&mut self) {
        if let Some(bias_unit) = &self.bias.unit {
            if bias_unit == &self.common_chan_attributes.source_range.unit {
                self.bias.value = self
                    .common_chan_attributes
                    .source_range
                    .limit(self.bias.value);
            } else {
                //default to 0.0 if source function has been changed
                self.bias.value = self.common_chan_attributes.source_range.limit(0.0);
                self.bias.unit = Some(self.common_chan_attributes.source_range.unit.clone());
            }
        } else {
            //TODO: handle error condition
            println!("bias.unit is None");
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
    fn migration_clamps_bias_and_removes_unsupported_attributes() {
        let old_device = device("MSMU60-2", 1);
        let new_device = device("MPSU50-2ST", 2);
        let mut channel = BiasChannel::new("bias1".to_string(), old_device);
        channel.bias.value = 200.0;

        channel.migrate_to_device(new_device.clone());

        assert_eq!(channel.common_chan_attributes.device_id, new_device._id);
        assert!((channel.bias.value - 50.1).abs() < 1e-9);
        assert!(channel.common_chan_attributes.source_limitv.is_none());
        assert!(channel.common_chan_attributes.sense_mode.is_none());
    }
}
