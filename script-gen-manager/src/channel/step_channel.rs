use std::any::Any;

use super::{
    common::CommonAttributes,
    default_channel::{Channel, ChannelAttributes},
};
use crate::device::SmuDevice;

#[derive(Debug, Clone)]
pub struct StepChannel {
    pub chan_attributes: ChannelAttributes,
    pub common_attributes: CommonAttributes,
}

impl Channel for StepChannel {
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

impl StepChannel {
    pub fn new(device: SmuDevice) -> Self {
        StepChannel {
            chan_attributes: ChannelAttributes::new("Sweep", "Step_Smu", device),
            common_attributes: CommonAttributes::new(),
        }
    }
}
