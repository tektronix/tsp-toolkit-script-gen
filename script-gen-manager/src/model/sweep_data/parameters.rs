use serde::{Deserialize, Serialize};

/// The `ParameterInt` struct represents an integer parameter.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParameterInt {
    pub id: String,
    pub value: i32,
}

/// The `ParameterFloat` struct represents a floating-point parameter.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParameterFloat {
    pub id: String,
    pub value: f64,
    pub unit: Option<String>,
}

/// The `ParameterString` struct represents a selection (drop-down) parameter.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParameterString {
    pub id: String,
    pub value: String,
    pub range: Vec<String>,
}

impl ParameterInt {
    pub fn new(id: &str, value: i32) -> Self {
        ParameterInt {
            id: id.to_string(),
            value,
        }
    }

    pub fn limit(&mut self, min: i32, max: i32) {
        if self.value >= min && self.value <= max {
            return;
        } else if self.value < min {
            self.value = min
        } else {
            self.value = max
        }
    }
}

impl ParameterFloat {
    pub fn new(id: &str, value: f64, unit: Option<String>) -> Self {
        ParameterFloat {
            id: id.to_string(),
            value,
            unit,
        }
    }

    pub fn limit(&mut self, min: f64, max: f64) {
        if self.value >= min && self.value <= max {
            return;
        } else if self.value < min {
            self.value = min
        } else {
            self.value = max
        }
    }
}

impl ParameterString {
    pub fn new(id: &str) -> Self {
        ParameterString {
            id: id.to_string(),
            value: String::new(),
            range: Vec::new(),
        }
    }
}
