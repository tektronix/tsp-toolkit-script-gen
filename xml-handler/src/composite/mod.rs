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

#[derive(Debug, Clone)]
pub struct SubstitutionScope {
    pub substitutions: Vec<Substitute>,
    pub parent: Option<Box<Self>>,
}

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
    /// The inherited substitution scope, if any.
    pub parent: Option<Box<SubstitutionScope>>,
}

impl Composite {
    const fn new(
        name: String,
        type_: Option<String>,
        indent: i32,
        repeat: String,
        conditions: Vec<Condition>,
        substitutions: Vec<Substitute>,
        sub_children: Vec<IncludeResult>,
    ) -> Self {
        Self {
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

    /// Parse the XML found in the `reader` to a [`Composite`]
    ///
    /// # Errors
    /// Errors may occur if parsing fails
    pub fn parse_composite<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Self> {
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
                    type_ = Some(String::from_utf8_lossy(attr.value.as_ref()).to_string());
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
                    repeat = String::from_utf8_lossy(attr.value.as_ref()).to_string();
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
                    return Ok(Self::new(
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
                self.evaluate(script_buffer, val_replacement_map);
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

        let temp = if index.is_none() || (index.unwrap() + 1) == symbol.len() {
            symbol.to_string()
        } else {
            let index = index.unwrap_or_else(|| symbol.len() - 1);
            let scope = &symbol[..index];

            val_replacement_map
                .get(&(scope.to_string() + ":"))
                .map_or_else(String::new, |val_arr| {
                    format!("{val_arr}{}", &symbol[index..])
                })
        };

        val_replacement_map
            .get(&temp)
            .map_or_else(String::new, std::clone::Clone::clone)
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
        let parent_scope = SubstitutionScope {
            substitutions: self.substitutions.clone(),
            parent: self.parent.clone(),
        };
        for res in &mut self.sub_children {
            match res {
                IncludeResult::Snippet(snippet) => {
                    snippet.parent = Some(Box::new(parent_scope.clone()));
                    snippet.to_script(script_buffer, val_replacement_map);
                }

                IncludeResult::Composite(composite) => {
                    composite.parent = Some(Box::new(parent_scope.clone()));
                    composite.to_script(script_buffer, val_replacement_map);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CommonChunk, Composite};
    use crate::group::IncludeResult;
    use crate::resources::MP5000_SWEEP_XML;
    use quick_xml::{events::Event, Reader};
    use script_aggregator::script_buffer::ScriptBuffer;
    use std::collections::HashMap;

    fn parse_composite(xml: &str) -> Composite {
        let mut reader = Reader::from_str(xml);
        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer).unwrap() {
                Event::Start(element) if element.name().as_ref() == b"composite" => {
                    return Composite::parse_composite(&mut reader, element.attributes()).unwrap();
                }
                Event::Eof => panic!("composite element not found"),
                _ => buffer.clear(),
            }
        }
    }

    fn find_repeated_composite<'a>(
        children: &'a [IncludeResult],
        repeat: &str,
    ) -> Option<&'a Composite> {
        for child in children {
            if let IncludeResult::Composite(composite) = child {
                if composite.repeat == repeat {
                    return Some(composite);
                }
                if let Some(found) = find_repeated_composite(&composite.sub_children, repeat) {
                    return Some(found);
                }
            }
        }
        None
    }

    #[test]
    fn repeated_composite_applies_parent_substitutions_for_six_values() {
        let mut composite = parse_composite(
            r#"<composite repeat="ITEMS">
                <substitute name="ITEMS:VALUE">%VALUE%</substitute>
                <composite>
                    <snippet>value=%VALUE%</snippet>
                </composite>
            </composite>"#,
        );
        let mut values = HashMap::from([(
            "ITEMS".to_string(),
            "item1,item2,item3,item4,item5,item6".to_string(),
        )]);
        for index in 1..=6 {
            values.insert(format!("item{index}:VALUE"), index.to_string());
        }
        let mut script_buffer = ScriptBuffer::new();

        composite.to_script(&mut script_buffer, &values);

        assert_eq!(
            script_buffer.to_string(),
            "value=1\nvalue=2\nvalue=3\nvalue=4\nvalue=5\nvalue=6\n"
        );

        let IncludeResult::Composite(child) = &composite.sub_children[0] else {
            panic!("expected nested composite");
        };
        let IncludeResult::Snippet(snippet) = &child.sub_children[0] else {
            panic!("expected nested snippet");
        };
        let mut scope = snippet.parent.as_deref();
        let mut scope_depth = 0;
        while let Some(current_scope) = scope {
            scope_depth += 1;
            scope = current_scope.parent.as_deref();
        }
        assert_eq!(scope_depth, 2);
    }

    #[test]
    fn production_sweep_composite_retains_only_substitution_scopes() {
        let root = parse_composite(&MP5000_SWEEP_XML.to_string());
        let mut sweep = find_repeated_composite(&root.sub_children, "SWEEP-DEVICE")
            .expect("production sweep repeat not found")
            .clone();
        let mut values = HashMap::from([(
            "SWEEP-DEVICE".to_string(),
            "sweep1,sweep2,sweep3,sweep4,sweep5,sweep6".to_string(),
        )]);
        for index in 1..=6 {
            values.insert(format!("sweep{index}:MODE"), "LIN".to_string());
        }
        let mut script_buffer = ScriptBuffer::new();

        sweep.to_script(&mut script_buffer, &values);

        assert_ne!(script_buffer.to_string(), "");
        for child in &sweep.sub_children {
            let parent = match child {
                IncludeResult::Snippet(snippet) => snippet.parent.as_deref(),
                IncludeResult::Composite(composite) => composite.parent.as_deref(),
            }
            .expect("child substitution scope not set");
            assert_eq!(parent.substitutions.len(), sweep.substitutions.len());
            assert!(parent.parent.is_none());
        }
    }
}
