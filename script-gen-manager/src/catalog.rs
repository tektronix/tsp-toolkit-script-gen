use std::collections::HashMap;

use xml_handler::{generic_parser, group::Group};

/// Represents a catalog of function metadata.
#[derive(Debug, Clone)]
pub struct Catalog {
    /// A map of function metadata, keyed by function type (e.g., Initialize, Finalize, etc.)
    pub function_metadata_map: HashMap<String, Group>,
}

impl Catalog {
    #[must_use]
    pub fn new() -> Self {
        Self {
            function_metadata_map: HashMap::new(),
        }
    }

    /// Refreshes the function metadata by parsing XML data.
    ///
    /// This method updates the `function_metadata_map` with the parsed XML data.
    pub fn refresh_function_metadata(&mut self) {
        if let Ok(res) = generic_parser::parse_xml() {
            for item in res {
                self.function_metadata_map.insert(item.type_.clone(), item);
            }
        } else {
            //eprintln!("Error: {:?}", e);
            //return Err(e.into());
        }
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}
