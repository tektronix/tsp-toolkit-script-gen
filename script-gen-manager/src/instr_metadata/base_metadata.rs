use std::collections::HashMap;
use std::fmt::Debug;

use phf::phf_map;

/// A static map that associates Trebuchet model numbers with their types.
pub static MODEL_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "MP5103" => "Mainframe",
    "MPSU50-2ST" => "Psu",
    "MSMU60-2" => "Smu",
};

pub trait Metadata: Debug + Clone {
    fn get_option(&self, key: &str) -> Option<&Vec<&'static str>>;
    fn get_range(&self, key: &str) -> Option<(f64, f64)>;
    fn get_default(&self, key: &str) -> Option<&'static str>;
}

/// A struct that holds base metadata information common to all Trebuchet instruments.
#[derive(Debug, Clone)]
pub struct BaseMetadata {
    options: HashMap<&'static str, Vec<&'static str>>,
    ranges: HashMap<String, (f64, f64)>,
    defaults: HashMap<&'static str, &'static str>,
}

impl BaseMetadata {
    pub const STYLE_LIN: &'static str = "LIN";
    pub const STYLE_LOG: &'static str = "LOG";
    //TODO: verify this value for Trebuchet
    pub const EPSILON: f64 = 1e-9;
    //TODO: verify this value for Trebuchet
    pub const MIN_LOG_VALUE: f64 = 1e-12;

    pub const OFF_VALUE: &'static str = "OFF";
    pub const ON_VALUE: &'static str = "ON";
    pub const ONCE_VALUE: &'static str = "ONCE";
    pub const AUTO_VALUE: &'static str = "AUTO";
    pub const USER_DEFINED_VALUE: &'static str = "USER DEFINED";
    pub const MOVING_AVG: &'static str = "MOVING AVG";
    pub const REPEAT_AVG: &'static str = "REPEAT AVG";
    pub const FUNCTION_VOLTAGE: &'static str = "Voltage";
    pub const FUNCTION_CURRENT: &'static str = "Current";
    pub const FUNCTION_IV: &'static str = "Current,Voltage";

    pub const UNIT_VOLTS: &'static str = "V";
    pub const UNIT_AMPERES: &'static str = "A";
    pub const UNIT_SECONDS: &'static str = "s";

    pub fn new() -> Self {
        let mut options = HashMap::new();
        let ranges = HashMap::new();
        let defaults = HashMap::new();

        //timing: source or measure delay type
        options.insert(
            "timing.delay.type",
            vec![
                BaseMetadata::OFF_VALUE,
                BaseMetadata::AUTO_VALUE,
                BaseMetadata::USER_DEFINED_VALUE,
            ],
        );

        BaseMetadata {
            options,
            ranges,
            defaults,
        }
    }

    pub fn add_option(&mut self, key: &'static str, value: Vec<&'static str>) {
        self.options.insert(key, value);
    }

    pub fn add_range(&mut self, key: String, min: f64, max: f64) {
        self.ranges.insert(key, (min, max));
    }

    pub fn add_default(&mut self, key: &'static str, value: &'static str) {
        self.defaults.insert(key, value);
    }
}

impl Metadata for BaseMetadata {
    /// Retrieves an option based on the provided key.
    fn get_option(&self, key: &str) -> Option<&Vec<&'static str>> {
        self.options.get(key)
    }

    /// Retrieves a range based on the provided key.
    fn get_range(&self, key: &str) -> Option<(f64, f64)> {
        self.ranges.get(key).cloned()
    }

    fn get_default(&self, key: &str) -> Option<&'static str> {
        self.defaults.get(key).cloned()
    }
}

impl Default for BaseMetadata {
    /// Provides a default instance of `BaseMetadata`.
    fn default() -> Self {
        BaseMetadata::new()
    }
}
