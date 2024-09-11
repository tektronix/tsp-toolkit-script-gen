use std::{
    any::Any,
    collections::HashMap,
    fs::File,
    io::{self, BufRead, Cursor, Write},
};

use quick_xml::{events::Event, name::QName, Reader};
use script_aggregator::script_buffer::ScriptBuffer;

use crate::{
    composite::CommonChunk,
    condition::Condition,
    error::{Result, XMLHandlerError},
    substitute::Substitute,
};

#[derive(Debug, Clone)]
pub struct Snippet {
    pub name: String,
    pub repeat: String,
    pub indent: i32,
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
            indent: 0,
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
                Ok(Event::Text(mut e)) => {
                    // Capture the text content inside the <snippet> tag
                    e.inplace_trim_start();
                    e.inplace_trim_end();
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

    pub fn evaluate(
        &self,
        script_buffer: &mut ScriptBuffer,
        val_replacement_map: &std::collections::HashMap<String, String>,
    ) {
        let mut temp_code_snippet = self.code_snippet.clone();

        for sub in self.substitutions.iter() {
            let to_val = self.lookup(val_replacement_map, &sub.name);
            temp_code_snippet = temp_code_snippet.replace(&sub.value, &to_val);
        }

        for key in val_replacement_map.keys() {
            let from_val = format!("%{}%", key);
            temp_code_snippet =
                temp_code_snippet.replace(&from_val, val_replacement_map.get(key).unwrap());
        }
        self.insert(script_buffer, temp_code_snippet);
    }

    fn insert(&self, script_buffer: &mut ScriptBuffer, temp_code: String) {
        // Create a cursor to read the string as bytes
        let cursor = Cursor::new(temp_code);
        let reader = io::BufReader::new(cursor);

        // Read the input string line by line
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    script_buffer.append(line);
                }
                Err(e) => {
                    //TODO: Add error handling
                    eprintln!("Error reading line: {}", e);
                }
            }
        }
    }
}

impl CommonChunk for Snippet {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_repeat(&self) -> &str {
        self.repeat.as_str()
    }

    fn get_indent(&self) -> i32 {
        self.indent
    }

    fn get_conditions(&self) -> &Vec<Condition> {
        &self.conditions
    }

    fn evaluate(&self, script_buffer: &mut ScriptBuffer, val_replacement_map: &HashMap<String, String>) {
        self.evaluate(script_buffer, val_replacement_map);
    }
}
