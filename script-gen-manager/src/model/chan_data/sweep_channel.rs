use serde::{Deserialize, Serialize};

use super::start_stop_channel::StartStopChannel;
use crate::device::Device;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepChannel {
    pub start_stop_channel: StartStopChannel,
}

impl SweepChannel {
    pub fn new(chan_name: String, device: Device, sweep_points: i32) -> Self {
        let mut step_channel = SweepChannel {
            start_stop_channel: StartStopChannel::new(chan_name, device),
        };

        step_channel.start_stop_channel.set_defaults(sweep_points);
        step_channel
    }
}
