use std::collections::HashMap;

use script_templates::FunctionTemplate;

/// Represents a catalog of function templates.
#[derive(Debug, Clone)]
pub struct Catalog {
    /// A map of function templates, keyed by function type (e.g., Initialize, Finalize, etc.)
    pub function_metadata_map: HashMap<String, FunctionTemplate>,
}

impl Catalog {
    /// The function types whose templates are included in generated scripts. Templates for
    /// Initialize, Finalize and DataReport exist but are not yet enabled.
    const ENABLED_FUNCTIONS: &'static [&'static str] = &["Sweep"];

    #[must_use]
    pub fn new() -> Self {
        Self {
            function_metadata_map: HashMap::new(),
        }
    }

    /// Refreshes the function metadata by loading the function templates.
    ///
    /// This method updates the `function_metadata_map` with the enabled function templates.
    pub fn refresh_function_metadata(&mut self) {
        match script_templates::load_templates() {
            Ok(templates) => {
                for template in templates
                    .into_iter()
                    .filter(|t| Self::ENABLED_FUNCTIONS.contains(&t.type_()))
                {
                    self.function_metadata_map
                        .insert(template.type_().to_string(), template);
                }
            }
            Err(e) => {
                eprintln!("Error loading function templates: {e}");
            }
        }
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}
