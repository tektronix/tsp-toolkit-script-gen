use quick_xml::events::attributes;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum XMLHandlerError {
    /// An IO error occurred
    #[error("IO error occurred: {source}")]
    IOError {
        /// The original `[std::io::Error]`
        #[from]
        source: std::io::Error,
    },

    /// An error occurred when parsing attributes
    #[error("Attribute parsing error: {source}")]
    AttributeError {
        /// The original error
        #[from]
        source: attributes::AttrError,
    },

    /// An error occurred when parsing XML file path attribute
    #[error("XML file path attribute error: {name}")]
    ResourceNotFoundError {
        /// The name of the resource that was not found
        name: String,
    },
}

pub type Result<T> = std::result::Result<T, XMLHandlerError>;
