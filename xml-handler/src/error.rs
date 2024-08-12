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

    /// An error occurred when parsing XML
    #[error("XML parsing error: {source}")]
    ParseError {
        /// The original error
        #[from]
        source: quick_xml::Error,
    },

    /// When the xml file supplied is not recognized
    #[error("Unknown XML file error: {file_name}")]
    UnknownXMLFileError {
        /// The name of the file
        file_name: String,
    }
}

pub type Result<T> = std::result::Result<T, XMLHandlerError>;
