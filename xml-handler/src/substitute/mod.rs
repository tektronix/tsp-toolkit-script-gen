use crate::error::{Result, XMLHandlerError};
use quick_xml::{events::Event, name::QName, Reader};

#[derive(Debug)]
pub struct Substitute {
    pub name: String,
    pub value: String,
}

impl Substitute {
    /// Function to create a new instance of [`Substitute`]
    fn new(name: String, value: String) -> Self {
        Substitute { name, value }
    }

    pub fn parse_substitute<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Substitute> {
        let mut name = String::new();
        let mut value = String::new();

        let mut buf: Vec<u8> = Vec::new();

        for attr in attributes {
            let attr = attr?;
            match attr.key {
                QName(b"name") => name = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                //QName(b"value") => value = String::from_utf8(attr.value.into_owned()).unwrap(),
                _ => {}
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
                    Err(_) => {}
                }
            }
            _ => (),
        }

        Ok(Substitute::new(name, value))
    }
}
