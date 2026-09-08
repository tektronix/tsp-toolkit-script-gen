use crate::error::{Result, XMLHandlerError};
use quick_xml::{events::Event, name::QName, Reader};

/// Represents the substitute tag in the XML data.
#[derive(Debug, Clone)]
pub struct Substitute {
    /// The substitute name.
    pub name: String,
    /// The substitute pattern.
    pub value: String,
}

impl Substitute {
    /// Function to create a new instance of [`Substitute`]
    const fn new(name: String, value: String) -> Self {
        Self { name, value }
    }

    /// Parse the XML in the `reader` and produce a [`Substitute`].
    ///
    /// # Errors
    /// Parsing errors may occur
    pub fn parse_substitute<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Self> {
        let mut name = String::new();
        let mut value = String::new();

        let mut buf: Vec<u8> = Vec::new();

        for attr in attributes {
            let attr = attr?;
            if attr.key == QName(b"name") {
                name = String::from_utf8_lossy(attr.value.as_ref()).to_string();
            }
        }

        match reader.read_event_into(&mut buf) {
            Err(e) => {
                eprintln!("Error at position {}: {:?}", reader.error_position(), e);
                return Err(XMLHandlerError::ParseError { source: e });
            }
            Ok(Event::Text(e)) => {
                // Capture the text content inside the <substitute> tag
                match e.unescape() {
                    Ok(text) => value = text.to_string(),
                    Err(e) => {
                        eprintln!("Error reading substitute value: {e:?}");
                        return Err(XMLHandlerError::ParseError { source: e });
                    }
                }
            }
            _ => (),
        }

        Ok(Self::new(name, value))
    }
}
