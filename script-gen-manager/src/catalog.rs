use std::collections::HashMap;

use xml_handler::{generic_parser, group::Group};

/// Represents a catalog of function metadata.
#[derive(Debug, Clone)]
pub struct Catalog {
    /// A map of function metadata, keyed by function type (e.g., Initialize, Finalize, etc.)
    pub function_metadata_map: HashMap<String, Group>,
}

impl Catalog {
    pub fn new() -> Self {
        Catalog {
            function_metadata_map: HashMap::new(),
        }
    }

    /// Refreshes the function metadata by parsing XML data.
    ///
    /// This method updates the `function_metadata_map` with the parsed XML data.
    pub fn refresh_function_metadata(&mut self) {
        match generic_parser::parse_xml() {
            Ok(res) => {
                for item in res {
                    self.function_metadata_map.insert(item.type_.clone(), item);
                }
            }
            Err(e) => {
                //eprintln!("Error: {:?}", e);
                //return Err(e.into());
            }
        }
    }
}
