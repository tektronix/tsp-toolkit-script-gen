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
}

pub type Result<T> = std::result::Result<T, XMLHandlerError>;
