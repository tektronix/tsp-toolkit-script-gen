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
    pub fn new(chan_name: String, device: Device) -> Self {
        let mut bias_channel = BiasChannel {
            common_chan_attributes: CommonChanAttributes::new(chan_name, device),
            bias: ParameterFloat::new("bias", 0.0, Some(BaseMetadata::UNIT_VOLTS.to_string())),
        };

        // Call set_defaults on the common_chan_attributes field
        bias_channel.common_chan_attributes.set_defaults();

        bias_channel
    }

    pub fn evaluate(&mut self) {
        self.common_chan_attributes.evaluate();
        self.determine_bias_value();
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
