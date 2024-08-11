use crate::error::Result;
use quick_xml::{events::Event, name::QName, Reader};

#[derive(Debug)]
pub struct Condition {
    pub name: String,
    op: String,
    pub value: String,
}

impl Condition {
    fn new(name: String, op: String, value: String) -> Self {
        Condition { name, op, value }
    }

    pub fn parse_condition<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Condition> {
        let mut name = String::new();
        let mut op = String::new();
        let mut value = String::new();

        let mut buf: Vec<u8> = Vec::new();

        for attr in attributes {
            let attr = attr?;
            match attr.key {
                QName(b"name") => name = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                QName(b"op") => op = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                _ => {}
            }
        }

        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(e)) => {
                // Capture the text content inside the <substitute> tag
                match e.unescape() {
                    Ok(text) => value = text.to_string(),
                    Err(_) => {}
                }
            }
            _ => (),
        }

        Ok(Condition::new(name, op, value))
    }
}
