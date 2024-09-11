use core::f64;

#[derive(Debug, Clone)]
pub enum RangeValue {
    String(String),
    Number(f64),
    None,
}

#[derive(Debug, Clone)]
pub struct ChannelRange {
    min: f64,
    max: f64,
    range: RangeValue,
}

impl ChannelRange {
    pub fn new() -> Self {
        ChannelRange {
            min: f64::NAN,
            max: f64::NAN,
            range: RangeValue::String("auto".to_string()),
        }
    }

    pub fn get_range(&self) -> &RangeValue {
        &self.range
    }

    pub fn set_range(&mut self, range: RangeValue) {
        self.range = range;
    }
}
