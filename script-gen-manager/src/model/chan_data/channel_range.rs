use serde::{Deserialize, Serialize};

use crate::instr_metadata::base_metadata::BaseMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRange {
    pub range: Vec<String>,
    pub value: String,
    #[serde(skip)]
    pub unit: String,
    #[serde(skip)]
    pub min: f64,
    #[serde(skip)]
    pub max: f64,
    #[serde(skip)]
    pub overrange_scale: f64,
}

impl ChannelRange {
    pub fn new() -> Self {
        ChannelRange {
            range: Vec::new(),
            value: String::new(),
            unit: String::new(),
            min: 0.0,
            max: 0.0,
            overrange_scale: 1.0,
        }
    }

    pub fn set_min(&mut self, min: f64) {
        self.min = min;
    }

    pub fn set_max(&mut self, max: f64) {
        self.max = max;
    }

    pub fn set_overrange_scale(&mut self, scale: f64) {
        self.overrange_scale = scale;
    }

    pub fn limit(&mut self, value: f64) -> f64 {
        let result = value;
        if self.value == "AUTO" {
            if result < self.min {
                return self.min;
            } else if result > self.max {
                return self.max;
            }
        } else {
            let scaled_value = self.get_scaled_value();
            if let Some(scaled_value) = scaled_value {
                let overrange_scaled_value = scaled_value * self.overrange_scale;
                if result < -overrange_scaled_value {
                    return -overrange_scaled_value;
                } else if result > overrange_scaled_value {
                    return overrange_scaled_value;
                }
            }
        }
        //TODO: error handling?
        result
    }

    pub fn get_scaled_value(&self) -> Option<f64> {
        // Extract the numeric part and the prefix+unit from the value string
        let mut numeric_part = String::new();
        let mut suffix_part = String::new();

        for c in self.value.chars() {
            if c.is_ascii_digit() || c == '.' {
                numeric_part.push(c);
            } else {
                suffix_part.push(c);
            }
        }

        // Trim the suffix_part to remove any leading or trailing whitespace
        let suffix_part = suffix_part.trim();

        // Parse the numeric part as f64
        let numeric_value: f64 = numeric_part.parse().ok()?;

        // Extract the prefix (e.g., "m", "k")
        let prefix = self.extract_prefix(suffix_part);

        // Determine the scaling factor based on the prefix
        let scaling_factor = match prefix.as_str() {
            "f" => 1e-15,       // femto
            "p" => 1e-12,       // pico
            "n" => 1e-9,        // nano
            "\u{00B5}" => 1e-6, // micro
            "m" => 1e-3,        // milli
            "" => 1.0,          // no prefix
            "k" => 1e3,         // kilo
            "M" => 1e6,         // mega
            _ => return None,   // Unknown prefix
        };

        // Calculate the scaled value
        Some(numeric_value * scaling_factor)
    }

    /// Extracts the prefix from a suffix string (e.g., "mV" -> "m").
    fn extract_prefix(&self, suffix: &str) -> String {
        let mut res = String::new();
        if suffix.is_empty() {
            return res;
        }

        // Check if the suffix ends with the expected unit
        if suffix.ends_with(&self.unit) {
            let prefix_length = suffix.len() - self.unit.len();
            let prefix = &suffix[..prefix_length]; // Extract the prefix
            res = prefix.to_string();
        }

        res
    }

    pub fn is_range_auto(&self) -> bool {
        self.value == BaseMetadata::AUTO_VALUE
    }

    pub fn is_range_follow_limiti(&self) -> bool {
        self.value == BaseMetadata::RANGE_FOLLOW_LIMITI
    }
}

impl Default for ChannelRange {
    fn default() -> Self {
        ChannelRange::new()
    }
}
