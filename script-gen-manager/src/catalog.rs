use std::collections::HashMap;

use xml_handler::{generic_parser, group::Group};

#[derive(Debug, Clone)]
pub struct Catalog {
    pub function_metadata_map: HashMap<String, Group>,
}

impl Catalog {
    pub fn new() -> Self {
        Catalog {
            function_metadata_map: HashMap::new(),
        }
    }

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
