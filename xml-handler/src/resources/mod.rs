use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::Display;

lazy_static! {
    static ref RESOURCE_MAP: HashMap<&'static str, &'static Resource> = {
        let mut m = HashMap::new();
        m.insert("DEFAULT_FUNC_METADATA", &DEFAULT_FUNC_METADATA);
        m.insert("INITIALIZE_XML", &INITIALIZE_XML);
        m.insert("SUPPORT_XML", &SUPPORT_XML);
        m.insert("DEFAULT_LIMITS_XML", &DEFAULT_LIMITS_XML);
        m
    };
}

use crate::VERSION;
const VERSION_REPLACE: &str = "!<!<VERSION>!>!";

pub const DEFAULT_FUNC_METADATA: Resource = Resource {
    source: include_str!("./DefaultFunctionMetaData.xml"),
};

pub const INITIALIZE_XML: Resource = Resource {
    source: include_str!("./Initialize.xml"),
};

pub const SUPPORT_XML: Resource = Resource {
    source: include_str!("./Support.xml"),
};

pub const DEFAULT_LIMITS_XML: Resource = Resource {
    source: include_str!("./sweep/DefaultLimits.xml"),
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
