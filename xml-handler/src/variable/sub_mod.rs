use crate::error::{Result, XMLHandlerError};
use quick_xml::{events::Event, name::QName, Reader};

#[derive(Debug)]
pub struct Reference {
    pub id: String,
    default: String,
    useall: String,
    value: String,
}

#[derive(Debug)]
pub struct Constraint {
    pub min: f64,
    pub max: f64,
}

impl Reference {
    fn new(id: String, default: String, useall: String, value: String) -> Self {
        Reference {
            id,
            default,
            useall,
            value,
        }
    }

    pub fn parse_reference_attr_only(
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Reference> {
        let mut id = String::new();
        let default = String::new();
        let useall = String::new();
        let value = String::new();

        for attr in attributes {
            let attr = attr?;
            match attr.key {
                QName(b"id") => id = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                _ => {}
            }
        }

        Ok(Reference::new(id, default, useall, value))
    }

    pub fn parse_reference<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Reference> {
        let mut buf: Vec<u8> = Vec::new();

        let mut id = String::new();
        let mut default = String::new();
        let mut useall = String::new();
        let mut value = String::new();

        for attr in attributes {
            let attr = attr?;
            match attr.key {
                QName(b"id") => id = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                QName(b"useall") => {
                    useall = String::from_utf8_lossy(attr.value.as_ref()).to_string()
                }
                QName(b"value") => value = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                _ => {}
            }
        }

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    eprintln!("Error at position {}: {:?}", reader.error_position(), e);
                    return Err(XMLHandlerError::ParseError { source: e });
                }
                Ok(Event::Text(e)) => {
                    default = e.unescape().unwrap().to_string();
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"reference" => {
                    break;
                }
                _ => {}
            }
        }

        Ok(Reference::new(id, default, useall, value))
    }
}

impl Constraint {
    pub fn new(min: f64, max: f64) -> Self {
        Constraint { min, max }
    }

    pub fn parse_constraint<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Constraint> {
        let mut buf: Vec<u8> = Vec::new();

        let mut min: f64 = 0.0;
        let mut max: f64 = 0.0; // Initialize max variable

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    eprintln!("Error at position {}: {:?}", reader.error_position(), e);
                    return Err(XMLHandlerError::ParseError { source: e });
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"min" => {
                    // Read text content of <min> tag
                    match reader.read_event_into(&mut buf) {
                        Ok(Event::Text(e)) => min = e.unescape().unwrap().parse().unwrap(),
                        _ => {}
                    }
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"max" => {
                    // Read text content of <max> tag
                    match reader.read_event_into(&mut buf) {
                        Ok(Event::Text(e)) => max = e.unescape().unwrap().parse().unwrap(),
                        _ => {}
                    }
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"constraints" => {
                    // Exit the loop when </constraints> is encountered
                    break;
                }
                _ => {}
            }
        }

        Ok(Constraint::new(min, max))
    }
}
