use thiserror::Error;

/// Define errors that originate from this crate
#[derive(Error, Debug)]
pub enum TemplateError {
    /// A template file is not valid YAML or is missing required fields
    #[error("error reading template file {file_name}: {source}")]
    YamlError {
        /// The name of the template file
        file_name: String,
        /// The original error
        source: serde_yaml_ng::Error,
    },

    /// A template or partial is not valid Handlebars
    #[error("error compiling template '{name}' in {file_name}: {source}")]
    CompileError {
        /// The name of the template file
        file_name: String,
        /// The name of the template or partial
        name: String,
        /// The original error
        source: Box<handlebars::TemplateError>,
    },

    /// A template could not be rendered with the provided context
    #[error("error rendering template: {source}")]
    RenderError {
        /// The original error
        #[from]
        source: handlebars::RenderError,
    },
}

pub type Result<T> = std::result::Result<T, TemplateError>;
