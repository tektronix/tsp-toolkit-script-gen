//! Handlebars templates for the functions that make up a generated TSP script.
//!
//! Each function (Sweep, Initialize, etc.) is described by a YAML file in `templates/`:
//!
//! ```yaml
//! id: SWEEP
//! type: Sweep
//! partials:          # optional, usable from body/aux as {{> name}}
//!   name: |
//!     ...
//! body: |            # the function's code
//!   ...
//! aux: |             # optional, code placed in the script preamble
//!   ...
//! ```
//!
//! Templates are rendered in strict mode with HTML escaping disabled, so every value referenced
//! by a template must be present in the rendering context.

use std::{collections::BTreeMap, fmt, sync::Arc};

use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

pub mod error;

use crate::error::{Result, TemplateError};

/// The template files embedded in this crate as `(file name, contents)`.
const TEMPLATE_FILES: &[(&str, &str)] = &[
    ("sweep.yaml", include_str!("../templates/sweep.yaml")),
    (
        "initialize.yaml",
        include_str!("../templates/initialize.yaml"),
    ),
    ("finalize.yaml", include_str!("../templates/finalize.yaml")),
    (
        "data_report.yaml",
        include_str!("../templates/data_report.yaml"),
    ),
];

const BODY: &str = "body";
const AUX: &str = "aux";

/// The contents of a template YAML file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateFile {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    #[serde(default)]
    partials: BTreeMap<String, String>,
    body: String,
    aux: Option<String>,
}

/// The compiled templates for a single script function (e.g., Sweep, Initialize, etc.)
#[derive(Clone)]
pub struct FunctionTemplate {
    /// The unique identifier for the function (e.g., SWEEP, INITIALIZE, etc.)
    id: String,
    /// The type of the function (e.g., Sweep, Initialize, etc.)
    type_: String,
    /// The compiled body, aux and partial templates.
    registry: Arc<Handlebars<'static>>,
}

impl FunctionTemplate {
    /// Parse and compile a template from the contents of a template YAML file.
    ///
    /// # Errors
    /// Returns an error if the YAML is malformed or if any of the templates are not valid
    /// Handlebars.
    pub fn parse(file_name: &str, yaml: &str) -> Result<Self> {
        let file: TemplateFile =
            serde_yaml_ng::from_str(yaml).map_err(|source| TemplateError::YamlError {
                file_name: file_name.to_string(),
                source,
            })?;

        let mut registry = Handlebars::new();
        registry.set_strict_mode(true);
        registry.register_escape_fn(handlebars::no_escape);

        let compile_error =
            |name: &str, source: handlebars::TemplateError| TemplateError::CompileError {
                file_name: file_name.to_string(),
                name: name.to_string(),
                source: Box::new(source),
            };

        for (name, partial) in &file.partials {
            registry
                .register_partial(name, partial)
                .map_err(|e| compile_error(name, e))?;
        }
        registry
            .register_template_string(BODY, &file.body)
            .map_err(|e| compile_error(BODY, e))?;
        if let Some(aux) = &file.aux {
            registry
                .register_template_string(AUX, aux)
                .map_err(|e| compile_error(AUX, e))?;
        }

        Ok(Self {
            id: file.id,
            type_: file.type_,
            registry: Arc::new(registry),
        })
    }

    /// The unique identifier for the function (e.g., SWEEP, INITIALIZE, etc.)
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The type of the function (e.g., Sweep, Initialize, etc.)
    #[must_use]
    pub fn type_(&self) -> &str {
        &self.type_
    }

    /// Render the function body with the given context.
    ///
    /// # Errors
    /// Returns an error if the template references a value missing from `context`.
    pub fn render_body<T: Serialize>(&self, context: &T) -> Result<String> {
        Ok(self.registry.render(BODY, context)?)
    }

    /// Render the auxiliary (preamble) code with the given context, if this function has any.
    ///
    /// # Errors
    /// Returns an error if the template references a value missing from `context`.
    pub fn render_aux<T: Serialize>(&self, context: &T) -> Result<Option<String>> {
        if self.registry.has_template(AUX) {
            Ok(Some(self.registry.render(AUX, context)?))
        } else {
            Ok(None)
        }
    }
}

impl fmt::Debug for FunctionTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionTemplate")
            .field("id", &self.id)
            .field("type_", &self.type_)
            .finish_non_exhaustive()
    }
}

/// Parse and compile all of the function templates embedded in this crate.
///
/// # Errors
/// Returns an error if any of the embedded templates fail to parse.
pub fn load_templates() -> Result<Vec<FunctionTemplate>> {
    TEMPLATE_FILES
        .iter()
        .map(|(file_name, yaml)| FunctionTemplate::parse(file_name, yaml))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn embedded_templates_compile() {
        let templates = load_templates().unwrap();
        let types: Vec<&str> = templates.iter().map(FunctionTemplate::type_).collect();
        assert_eq!(types, ["Sweep", "Initialize", "Finalize", "DataReport"]);
    }

    #[test]
    fn renders_body_aux_and_partials() {
        let yaml = r#"
id: TEST
type: Test
partials:
  item: |
    item {{name}}
body: |
  {{#each items}}
  {{> item}}
  {{/each}}
  done <{{value}}>
aux: |
  aux {{value}}
"#;
        let template = FunctionTemplate::parse("test.yaml", yaml).unwrap();
        let context = json!({"items": [{"name": "a"}, {"name": "b"}], "value": "\"x\""});

        assert_eq!(template.id(), "TEST");
        assert_eq!(template.type_(), "Test");
        assert_eq!(
            template.render_body(&context).unwrap(),
            "item a\nitem b\ndone <\"x\">\n"
        );
        assert_eq!(
            template.render_aux(&context).unwrap().as_deref(),
            Some("aux \"x\"\n")
        );
    }

    #[test]
    fn aux_is_optional() {
        let template = FunctionTemplate::parse("test.yaml", "id: T\ntype: T\nbody: x\n").unwrap();
        assert!(template.render_aux(&json!({})).unwrap().is_none());
    }

    #[test]
    fn literal_braces_next_to_values() {
        let yaml = r#"
id: T
type: T
body: |
  a = "((\{{ {{~tag~}} }}))"
  b = "{ {{~tag~}} }"
  c = "((\{{}}))"
"#;
        let template = FunctionTemplate::parse("test.yaml", yaml).unwrap();
        assert_eq!(
            template.render_body(&json!({"tag": "Data"})).unwrap(),
            "a = \"(({{Data}}))\"\nb = \"{Data}\"\nc = \"(({{}}))\"\n"
        );
    }

    #[test]
    fn missing_value_is_an_error() {
        let template =
            FunctionTemplate::parse("test.yaml", "id: T\ntype: T\nbody: '{{x}}'\n").unwrap();
        assert!(matches!(
            template.render_body(&json!({})),
            Err(TemplateError::RenderError { .. })
        ));
    }

    #[test]
    fn invalid_template_is_an_error() {
        let err =
            FunctionTemplate::parse("bad.yaml", "id: T\ntype: T\nbody: '{{#if x}}'\n").unwrap_err();
        assert!(matches!(err, TemplateError::CompileError { ref name, .. } if name == BODY));
    }

    #[test]
    fn invalid_yaml_is_an_error() {
        let err = FunctionTemplate::parse("bad.yaml", "id: T\nbody: x\n").unwrap_err();
        assert!(matches!(err, TemplateError::YamlError { .. }));
    }
}
