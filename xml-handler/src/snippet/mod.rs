use std::{
    any::Any,
    collections::HashMap,
    io::{self, BufRead, Cursor},
};

use quick_xml::{events::Event, name::QName, Reader};
use script_aggregator::script_buffer::ScriptBuffer;

use crate::{
    composite::{CommonChunk, Composite},
    condition::Condition,
    error::{Result, XMLHandlerError},
    substitute::Substitute,
};

/// Represents the snippet tag in the XML data.
#[derive(Debug, Clone)]
pub struct Snippet {
    /// The name of the snippet.
    pub name: String,
    /// The repeat attribute of the snippet.
    pub repeat: String,
    /// The indent level
    pub indent: i32,
    /// The actual code snippet.
    pub code_snippet: String,
    /// The substitutions associated with the snippet.
    pub substitutions: Vec<Substitute>,
    /// The conditions associated with the snippet.
    pub conditions: Vec<Condition>,
    /// The parent Composite, if any.
    pub parent: Option<Box<Composite>>,
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
            parent: None,
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

    /// Evaluates the snippet and inserts the resulting code into the script buffer.
    ///
    /// This method processes the substitutions in the snippet and its parent composites,
    /// replaces the placeholders in the code snippet with the corresponding values,
    /// and inserts the resulting code into the script buffer.
    ///
    /// # Arguments
    ///
    /// * `script_buffer` - A mutable reference to the script buffer.
    /// * `val_replacement_map` - A reference to the value replacement map.
    pub fn evaluate_snippet(
        &self,
        script_buffer: &mut ScriptBuffer,
        val_replacement_map: &std::collections::HashMap<String, String>,
    ) {
        let mut temp_code_snippet = self.code_snippet.clone();

        for sub in self.substitutions.iter() {
            let to_val = self.lookup(val_replacement_map, &sub.name);
            temp_code_snippet = temp_code_snippet.replace(&sub.value, &to_val);
        }

        let mut current_parent = self.parent.as_deref(); // Use as_deref to get Option<&Composite>
        while let Some(parent) = current_parent {
            for sub in parent.substitutions.iter() {
                let to_val = parent.lookup(val_replacement_map, &sub.name);
                temp_code_snippet = temp_code_snippet.replace(&sub.value, &to_val);
            }
            current_parent = parent.parent.as_deref(); // Use as_deref to get Option<&Composite>
        }

        self.insert(script_buffer, temp_code_snippet);
    }

    /// Inserts the given text into the script buffer.
    ///
    /// Insert the specified text into the script buffer one line at a time
    /// to ensure the target script uses the correct EOL sequence.
    ///
    /// # Arguments
    ///
    /// * `script_buffer` - A mutable reference to the script buffer.
    /// * `temp_code` - The text to be inserted into the script buffer.
    fn insert(&self, script_buffer: &mut ScriptBuffer, temp_code: String) {
        // Create a cursor to read the string as bytes
        let cursor = Cursor::new(temp_code);
        let reader = io::BufReader::new(cursor);

        // Read the input string line by line
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    script_buffer.body_append(line);
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

    fn evaluate(
        &mut self,
        script_buffer: &mut ScriptBuffer,
        val_replacement_map: &HashMap<String, String>,
    ) {
        self.evaluate_snippet(script_buffer, val_replacement_map);
    }
}
