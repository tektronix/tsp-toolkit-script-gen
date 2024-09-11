use std::any::Any;

use super::{
    common::CommonAttributes,
    default_channel::{Channel, ChannelAttributes},
};
use crate::device::SmuDevice;

#[derive(Debug, Clone)]
pub struct SweepChannel {
    pub chan_attributes: ChannelAttributes,
    pub common_attributes: CommonAttributes,
    pub pulse_bias: f64,
}

impl Channel for SweepChannel {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_device(&self) -> &SmuDevice {
        &self.chan_attributes.device
    }

    fn get_channel_attributes(&mut self) -> &mut ChannelAttributes {
        &mut self.chan_attributes
    }
}

impl SweepChannel {
    pub fn new(device: SmuDevice) -> Self {
        SweepChannel {
            chan_attributes: ChannelAttributes::new("Sweep", "Sweep_Smu", device),
            common_attributes: CommonAttributes::new(),
            pulse_bias: 0.0,
        }
    }
}
