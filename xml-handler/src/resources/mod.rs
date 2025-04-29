use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::Display;

lazy_static! {
    static ref RESOURCE_MAP: HashMap<&'static str, &'static Resource> = {
        let mut m = HashMap::new();
        m.insert("DEFAULT_FUNC_METADATA", &DEFAULT_FUNC_METADATA);
        m.insert("SWEEP_FUNC_METADATA", &SWEEP_FUNC_METADATA);
        m.insert("INITIALIZE_XML", &INITIALIZE_XML);
        m.insert("SUPPORT_XML", &SUPPORT_XML);
        m.insert("DEFAULT_LIMITS_XML", &DEFAULT_LIMITS_XML);
        m.insert("MP5000_SWEEP_XML", &MP5000_SWEEP_XML);
        m.insert("DEFAULT_SWEEP_CHUNK_XML", &DEFAULT_SWEEP_CHUNK_XML);
        m.insert("DATA_REPORT_XML", &DATA_REPORT_XML);
        m.insert("FINALIZE_XML", &FINALIZE_XML);
        m.insert("NO_STEP_AUTO_XML", &NO_STEP_AUTO_XML);
        m.insert("STEP_AUTO_XML", &STEP_AUTO_XML);
        m.insert("NO_STEP_FIXED_XML", &NO_STEP_FIXED_XML);
        m.insert("STEP_FIXED_XML", &STEP_FIXED_XML);
        m
    };
}

use crate::VERSION;
const VERSION_REPLACE: &str = "!<!<VERSION>!>!";

pub const DEFAULT_FUNC_METADATA: Resource = Resource {
    source: include_str!("./DefaultFunctionMetaData.xml"),
};

pub const SWEEP_FUNC_METADATA: Resource = Resource {
    source: include_str!("./SweepFunctionMetaData.xml"),
};

pub const INITIALIZE_XML: Resource = Resource {
    source: include_str!("./Initialize.xml"),
};

pub const SUPPORT_XML: Resource = Resource {
    source: include_str!("./Support.xml"),
};

pub const DATA_REPORT_XML: Resource = Resource {
    source: include_str!("./DataReport.xml"),
};

pub const FINALIZE_XML: Resource = Resource {
    source: include_str!("./Finalize.xml"),
};

pub const DEFAULT_LIMITS_XML: Resource = Resource {
    source: include_str!("./sweep/DefaultLimits.xml"),
};

pub const MP5000_SWEEP_XML: Resource = Resource {
    source: include_str!("./sweep/MP5000Sweep.xml"),
};

pub const DEFAULT_SWEEP_CHUNK_XML: Resource = Resource {
    source: include_str!("./sweep/DefaultSweepChunk.xml"),
};

pub const NO_STEP_AUTO_XML: Resource = Resource {
    source: include_str!("./sweep/NoStepAuto.xml"),
};

pub const STEP_AUTO_XML: Resource = Resource {
    source: include_str!("./sweep/StepAuto.xml"),
};

pub const NO_STEP_FIXED_XML: Resource = Resource {
    source: include_str!("./sweep/NoStepFixed.xml"),
};

pub const STEP_FIXED_XML: Resource = Resource {
    source: include_str!("./sweep/StepFixed.xml"),
};

/// A resource that can be used as-is
#[derive(Debug)]
pub struct Resource {
    /// The raw resource that can be used as-is
    source: &'static str,
}

impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source = self.source.replace(VERSION_REPLACE, VERSION);
        write!(f, "{source}")
    }
}

impl Resource {
    /// Function to match an input string against all defined Resource constants
    pub fn match_resource(input: &str) -> Option<&'static Resource> {
        RESOURCE_MAP.get(input).copied()
    }
}
