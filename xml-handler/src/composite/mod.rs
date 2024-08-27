use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use script_aggregator::script_buffer::ScriptBuffer;

use crate::error::{Result, XMLHandlerError};
use crate::group::{parse_include, ExternalFileResult, IncludeResult};
use crate::snippet::Snippet;
use crate::substitute::Substitute;

#[derive(Debug, Clone)]
pub struct Composite {
    pub name: String,
    pub type_: Option<String>,
    pub indent: i32,
    pub repeat: String,

    pub substitutions: Vec<Substitute>,
    pub sub_children: Vec<IncludeResult>,
}

impl Composite {
    fn new(
        name: String,
        type_: Option<String>,
        indent: i32,
        repeat: String,
        substitutions: Vec<Substitute>,
        sub_children: Vec<IncludeResult>,
    ) -> Self {
        Composite {
            name,
            type_,
            indent,
            repeat,
            substitutions,
            sub_children,
        }
    }

    pub fn parse_composite<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Composite> {
        let mut name = String::new();
        let mut type_: Option<String> = None;
        let mut indent = 0;
        let mut repeat = String::new();

        let mut substitutions: Vec<Substitute> = Vec::new();
        let mut sub_children: Vec<IncludeResult> = Vec::new();

        let mut buf: Vec<u8> = Vec::new();

        for attr in attributes {
            let attr = attr?;
            match attr.key {
                QName(b"name") => name = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                QName(b"type") => {
                    type_ = Some(String::from_utf8_lossy(attr.value.as_ref()).to_string())
                }
                QName(b"indent") => {
                    let attr_val = String::from_utf8_lossy(attr.value.as_ref()).to_string();
                    if attr_val == "default" {
                        indent = 4;
                    } else {
                        indent = 0;
                    }
                }
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
                Ok(Event::Start(e)) if e.name().as_ref() == b"substitute" => {
                    substitutions.push(Substitute::parse_substitute(reader, e.attributes())?);
                }
                Ok(Event::Empty(e)) if e.name().as_ref() == b"include" => {
                    let res = parse_include(e.attributes())?;
                    match res {
                        ExternalFileResult::Snippet(snippet) => {
                            sub_children.push(IncludeResult::Snippet(snippet));
                        }
                        ExternalFileResult::Composite(composite) => {
                            sub_children.push(IncludeResult::Composite(composite));
                        }
                        ExternalFileResult::Variables(_) => {
                            todo!();
                        }
                    }
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"snippet" => {
                    let res = Snippet::parse_snippet(reader, e.attributes())?;
                    sub_children.push(IncludeResult::Snippet(res));
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"composite" => {
                    let res = Self::parse_composite(reader, e.attributes())?;
                    sub_children.push(IncludeResult::Composite(res));
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"composite" => {
                    return Ok(Composite::new(
                        name,
                        type_,
                        indent,
                        repeat,
                        substitutions,
                        sub_children,
                    ));
                }

                _ => (),
            }
        }
    }

    pub fn to_script(
        &self,
        temp: &mut ScriptBuffer,
        val_replacement_map: &HashMap<String, String>,
    ) {
        if self.evaluate_conditions(val_replacement_map) {
            if self.indent > 0 {
                temp.change_indent(self.indent);
            }
            if self.repeat.is_empty() {
                for res in self.sub_children.iter() {
                    if let IncludeResult::Snippet(snippet) = res {
                        snippet.evaluate(temp, val_replacement_map);
                    }
                }
            } else {
                todo!();
            }
            if self.indent > 0 {
                temp.change_indent(-self.indent);
            }
        }
    }

    fn evaluate_conditions(&self, val_replacement_map: &HashMap<String, String>) -> bool {
        //TODO: required mainly for sweep model
        true
    }
}
