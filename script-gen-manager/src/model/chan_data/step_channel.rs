use super::start_stop_channel::StartStopChannel;
use crate::device::Device;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepChannel {
    pub start_stop_channel: StartStopChannel,
}

impl StepChannel {
    pub fn new(chan_name: String, device: Device, step_points: i32) -> Self {
        let mut step_channel = StepChannel {
            start_stop_channel: StartStopChannel::new(chan_name, device),
        };
        step_channel.start_stop_channel.set_defaults(step_points);
        step_channel
    }
}
