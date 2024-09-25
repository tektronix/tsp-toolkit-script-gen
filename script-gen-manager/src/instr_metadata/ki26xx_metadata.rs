#[derive(Debug, Clone)]
pub struct Ki26xxMetadata {}

impl Ki26xxMetadata {
    pub const AUTO_VALUE: &'static str = "AUTO";
    pub const SOURCE_MODE_HIGH_C: &'static str = "High-C";
    pub const STYLE_LIN: &'static str = "LIN";
    pub const FUNCTION_IV: &'static str = "current,voltage";
    pub const EPSILON: f64 = 1e-9;

    pub fn new() -> Self {
        Ki26xxMetadata {}
    }
}
