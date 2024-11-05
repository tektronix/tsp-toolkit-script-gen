use std::any::Any;
use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use script_aggregator::script_buffer::ScriptBuffer;

use crate::condition::Condition;
use crate::error::{Result, XMLHandlerError};
use crate::group::{parse_include, ExternalFileResult, IncludeResult};
use crate::snippet::Snippet;
use crate::substitute::Substitute;

/// Represents the composite tag in the XML data.
#[derive(Debug, Clone)]
pub struct Composite {
    /// The name of the composite.
    pub name: String,
    /// The type of the composite, if specified (e.g., aux).
    pub type_: Option<String>,
    /// The indentation level.
    pub indent: i32,
    /// The repeat attribute of the composite.
    pub repeat: String,

    /// The conditions associated with the composite.
    pub conditions: Vec<Condition>,
    /// The substitutions associated with the composite.
    pub substitutions: Vec<Substitute>,
    /// A composite can further contain more composites or snippets.
    pub sub_children: Vec<IncludeResult>,
    /// The parent Composite, if any.
    pub parent: Option<Box<Composite>>,
}

impl Composite {
    fn new(
        name: String,
        type_: Option<String>,
        indent: i32,
        repeat: String,
        conditions: Vec<Condition>,
        substitutions: Vec<Substitute>,
        sub_children: Vec<IncludeResult>,
    ) -> Self {
        Composite {
            name,
            type_,
            indent,
            repeat,
            conditions,
            substitutions,
            sub_children,
            parent: None,
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

        let mut conditions: Vec<Condition> = Vec::new();
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
                Ok(Event::Start(e)) if e.name().as_ref() == b"condition" => {
                    conditions.push(Condition::parse_condition(reader, e.attributes())?);
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
                        conditions,
                        substitutions,
                        sub_children,
                    ));
                }

                _ => (),
            }
        }
    }

    // pub fn to_script(
    //     &self,
    //     script_buffer: &mut ScriptBuffer,
    //     val_replacement_map: &HashMap<String, String>,
    // ) {
    //     if self.evaluate_conditions(val_replacement_map) {
    //         if self.indent > 0 {
    //             script_buffer.change_indent(self.indent);
    //         }
    //         if self.repeat.is_empty() {
    //             for res in self.sub_children.iter() {
    //                 if let IncludeResult::Snippet(snippet) = res {
    //                     snippet.evaluate(script_buffer, val_replacement_map);
    //                 }
    //             }
    //         } else {
    //             todo!();
    //         }
    //         if self.indent > 0 {
    //             script_buffer.change_indent(-self.indent);
    //         }
    //     }
    // }
}

pub trait CommonChunk {
    fn as_any(&self) -> &dyn Any;
    fn get_repeat(&self) -> &str;
    fn get_indent(&self) -> i32;
    fn get_conditions(&self) -> &Vec<Condition>;
    fn evaluate(
        &mut self,
        script_buffer: &mut ScriptBuffer,
        val_replacement_map: &HashMap<String, String>,
    );

    /// The composite/snippet instances are processed and the result is appended to the script buffer.
    ///
    /// # Arguments
    ///
    /// * `script_buffer` - A mutable reference to the script buffer.
    /// * `val_replacement_map` - A reference to the value replacement map.
    fn to_script(
        &mut self,
        script_buffer: &mut ScriptBuffer,
        val_replacement_map: &HashMap<String, String>,
    ) {
        // determines whether the composite/snippet should be included in the script
        if self.evaluate_conditions(val_replacement_map) {
            if self.get_indent() > 0 {
                script_buffer.change_indent(self.get_indent());
            }

            if self.get_repeat().is_empty() {
                self.evaluate(script_buffer, val_replacement_map)
            } else {
                let repeat_val = self.get_repeat();
                let active = repeat_val.to_owned() + ":";
                //TODO: Commenting below line for now, need to discuss if it is required
                //let obsolete = repeat_val.to_owned() + ".value"; // for backward compatibility

                let mut loop_count = 1;
                let loop_count_name = repeat_val.to_owned() + ".LOOP-COUNT";
                let list_arr = val_replacement_map.get(repeat_val);
                if let Some(list_arr) = list_arr {
                    let list = list_arr.split(',').collect::<Vec<&str>>();
                    for val in list {
                        let mut val_replacement_map1 = val_replacement_map.clone();
                        val_replacement_map1.insert(active.clone(), val.to_string());
                        //val_replacement_map.insert(obsolete.clone(), val.to_string());
                        val_replacement_map1
                            .insert(loop_count_name.clone(), loop_count.to_string());
                        self.evaluate(script_buffer, &val_replacement_map1);
                        loop_count += 1;
                    }
                }
            }

            if self.get_indent() > 0 {
                script_buffer.change_indent(-self.get_indent());
            }
        }
    }

    /// Evaluates the conditions (if any) associated with the composite/snippet.
    ///
    /// This method checks if the conditions specified in the composite/snippet
    /// are met based on the values in the value replacement map.
    ///
    /// # Arguments
    ///
    /// * `val_replacement_map` - A reference to the value replacement map.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the conditions are met.
    fn evaluate_conditions(&self, val_replacement_map: &HashMap<String, String>) -> bool {
        let mut include = true;
        let conditions = self.get_conditions();
        for condition in conditions {
            let op = &condition.op;
            let object = self.lookup(val_replacement_map, &condition.name);
            if object.is_empty() {
                //TODO: handle error
            }
            if "ne" == op {
                if condition.value == object {
                    include = false;
                    break;
                }
            } else if "gt" == op {
                let val1 = object.parse::<f64>().unwrap();
                let val2 = condition.value.parse::<f64>().unwrap();
                if val1 <= val2 {
                    include = false;
                    break;
                }
            } else if "ge" == op {
                let val1 = object.parse::<f64>().unwrap();
                let val2 = condition.value.parse::<f64>().unwrap();
                if val1 < val2 {
                    include = false;
                    break;
                }
            } else if "lt" == op {
                let val1 = object.parse::<f64>().unwrap();
                let val2 = condition.value.parse::<f64>().unwrap();
                if val1 >= val2 {
                    include = false;
                    break;
                }
            } else if "le" == op {
                let val1 = object.parse::<f64>().unwrap();
                let val2 = condition.value.parse::<f64>().unwrap();
                if val1 > val2 {
                    include = false;
                    break;
                }
            } else if "in" == op {
                // object must be in expression
                let expression = &condition.value;
                let token = object;
                let index = expression.find(&token);
                if let Some(index) = index {
                    // verify complete token match vs. partial token match
                    if (index > 0 && expression.chars().nth(index - 1) != Some(','))
                        || (index + token.len() < expression.len()
                            && expression.chars().nth(index + token.len()) != Some(','))
                    {
                        include = false;
                        break;
                    }
                } else {
                    include = false;
                    break;
                }
            } else if "regex" == op {
                //TODO: handle regex
            } else {
                // must be "e1" (==)
                if condition.value != object {
                    include = false;
                    break;
                }
            }
        }
        include
    }

    /// Looks up a value in the value replacement map based on the given symbol.
    ///
    /// e.g., The symbol "DEVICES:ASSIGN" has scope "DEVICES" and the value replacement map
    /// value of "DEVICES:" is extracted before doing the lookup - so if DEVICES: currently
    /// has the value "bias1" then the lookup becomes "bias1:ASSIGN".
    ///
    /// # Arguments
    ///
    /// * `val_replacement_map` - A reference to the value replacement map.
    /// * `symbol` - The symbol to look up.
    ///
    /// # Returns
    ///
    /// A string representing the looked-up value.
    fn lookup(&self, val_replacement_map: &HashMap<String, String>, symbol: &str) -> String {
        let index = symbol.find(':');
        let mut temp = "".to_string();

        if index.is_none() || (index.unwrap() + 1) == symbol.len() {
            temp = symbol.to_string();
        } else {
            let index = index.unwrap();
            let scope = &symbol[..index];

            if let Some(val_arr) = val_replacement_map.get(&(scope.to_string() + ":")) {
                temp = val_arr.to_string();
                temp += &symbol[index..];
            } else {
                //TODO: handle error
            }
        }

        match val_replacement_map.get(&temp) {
            Some(val) => val.clone(),
            None => {
                //handle error
                "".to_string()
            }
        }
    }
}

impl CommonChunk for Composite {
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
        let parent_clone = self.clone();
        for res in self.sub_children.iter_mut() {
            match res {
                IncludeResult::Snippet(snippet) => {
                    snippet.parent = Some(Box::new(parent_clone.clone()));
                    snippet.to_script(script_buffer, val_replacement_map)
                }

                IncludeResult::Composite(composite) => {
                    composite.parent = Some(Box::new(parent_clone.clone()));
                    composite.to_script(script_buffer, val_replacement_map)
                }
            }
        }
    }
}
