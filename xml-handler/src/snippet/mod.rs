use quick_xml::{events::Event, name::QName, Reader};

use crate::{
    condition::Condition,
    error::{Result, XMLHandlerError},
    substitute::Substitute,
};

#[derive(Debug)]
pub struct Snippet {
    pub name: String,
    pub repeat: String,
    pub code_snippet: String,
    pub substitutions: Vec<Substitute>,
    pub conditions: Vec<Condition>,
}

impl Snippet {
    /// Function to create a new instance of [`Snippet`]
    fn new(
        name: String,
        repeat: String,
        code_snippet: String,
        substitutions: Vec<Substitute>,
        conditions: Vec<Condition>,
    ) -> Self {
        Snippet {
            name,
            repeat,
            code_snippet,
            substitutions,
            conditions,
        }
    }

    pub fn parse_snippet<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Snippet> {
        let mut name = String::new();
        let mut repeat = String::new();

        let mut code_snippet = String::new();
        let mut substitutions: Vec<Substitute> = Vec::new();
        let mut conditions: Vec<Condition> = Vec::new();

        let mut buf: Vec<u8> = Vec::new();

        for attr in attributes {
            let attr = attr?;
            match attr.key {
                QName(b"name") => name = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                QName(b"repeat") => {
                    repeat = String::from_utf8_lossy(attr.value.as_ref()).to_string()
                }
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
                    // Capture the text content inside the <snippet> tag
                    match e.unescape() {
                        Ok(text) => {
                            code_snippet.push_str(&text);
                            // let mut file = File::create("C:\\Trebuchet\\Snippet.txt")?;
                            // file.write_all(code_snippet.as_bytes())?;
                        }
                        Err(e) => {
                            eprintln!("Error decoding text: {}", e);
                            return Err(XMLHandlerError::ParseError { source: e });
                        }
                    }
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"substitute" => {
                    substitutions.push(Substitute::parse_substitute(reader, e.attributes())?);
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"condition" => {
                    conditions.push(Condition::parse_condition(reader, e.attributes())?);
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"snippet" => {
                    return Ok(Snippet::new(
                        name,
                        repeat,
                        code_snippet,
                        substitutions,
                        conditions,
                    ));
                }
                _ => (),
            }
        }
    }
}
