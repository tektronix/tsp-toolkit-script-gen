use crate::instr_metadata::ki26xx_metadata::Ki26xxMetadata;

#[derive(Debug, Clone)]
pub struct CommonAttributes {
    pub start: f64,
    pub stop: f64,
    pub asymptote: f64,
    pub style: String,
}

impl CommonAttributes {
    pub const DEFAULT_START: f64 = 0.0;
    pub const DEFAULT_STOP: f64 = 1.0;

    pub fn new() -> Self {
        CommonAttributes {
            start: Self::DEFAULT_START,
            stop: Self::DEFAULT_STOP,
            asymptote: 0.0,
            style: Ki26xxMetadata::STYLE_LIN.to_string(),
        }
    }
}
