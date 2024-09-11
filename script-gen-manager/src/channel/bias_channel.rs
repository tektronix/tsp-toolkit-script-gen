use std::any::Any;

use super::default_channel::{Channel, ChannelAttributes};
use crate::device::SmuDevice;

#[derive(Debug, Clone)]
pub struct BiasChannel {
    pub chan_attributes: ChannelAttributes,
}

impl Channel for BiasChannel {
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

impl BiasChannel {
    pub fn new(device: SmuDevice) -> Self {
        let mut bias_chan = BiasChannel {
            chan_attributes: ChannelAttributes::new("Bias", "Bias_Smu", device),
        };
        bias_chan.set_defaults();
        bias_chan
    }
}
