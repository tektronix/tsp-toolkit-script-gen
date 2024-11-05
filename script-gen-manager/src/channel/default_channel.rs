use std::{any::Any, fmt::Debug};

use crate::device::SmuDevice;

use super::channel_range::ChannelRange;

pub trait Channel: Debug {
    fn as_any(&self) -> &dyn Any;
    /// Returns the name of the channel (e.g., "Bias_Smu", "Step_Smu" etc.)
    fn get_name(&self) -> &str;
    /// Returns the device associated with the channel.
    fn get_device(&self) -> &SmuDevice;
    /// Returns a mutable reference to the channel attributes.
    fn get_channel_attributes(&mut self) -> &mut ChannelAttributes;
    /// Returns the measurement function of the channel (e.g., "current", "voltage" etc.)
    fn get_measurement_function(&self) -> &str;

    fn set_defaults(&mut self) {
        //TODO: hardcoding this for now
        let chan_attributes = self.get_channel_attributes();
        chan_attributes.source_limit_i = 0.001;
        chan_attributes.source_limit_v = 1.0;
        chan_attributes.source_function = String::from("voltage");
        chan_attributes.measure_function = String::from("current");

        chan_attributes.sense_mode = String::from("Two-Wire");
        chan_attributes.source_mode = String::from("Normal");
    }
}

// Represents attributes common to bias, step and sweep channels.
#[derive(Debug, Clone)]
pub struct ChannelAttributes {
    pub func_name: &'static str,
    pub preferred_name: &'static str,
    pub device: SmuDevice,

    pub auto_range_enabled: bool,
    pub high_c_enabled: bool,
    pub pulse_enabled: bool,
    pub source_function: String,
    pub measure_function: String,
    pub source_limit_i: f64,
    pub source_limit_v: f64,
    pub sense_mode: String,
    pub source_mode: String,

    pub source_range: ChannelRange,
    pub measure_range: ChannelRange,
}

impl ChannelAttributes {
    pub fn new(func_name: &'static str, preferred_name: &'static str, device: SmuDevice) -> Self {
        ChannelAttributes {
            func_name,
            preferred_name,
            device,

            auto_range_enabled: true,
            high_c_enabled: false,
            pulse_enabled: false,
            source_function: String::from(""),
            measure_function: String::from(""),
            source_limit_i: 0.0,
            source_limit_v: 0.0,
            sense_mode: String::from(""),
            source_mode: String::from(""),

            source_range: ChannelRange::new(),
            measure_range: ChannelRange::new(),
        }
    }
}
