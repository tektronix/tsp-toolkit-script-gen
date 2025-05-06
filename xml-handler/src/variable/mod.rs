use quick_xml::{events::Event, name::QName, Reader};
mod sub_mod;

use crate::error::{Result, XMLHandlerError};
pub use sub_mod::{Constraint, Reference};

/// Represents a variables tag in the XML data.
#[derive(Debug, Clone)]
pub struct Variables {
    /// A list of variables associated with the variables tag.
    pub variable_array: Vec<Variable>,
}

/// Represents a variable tag in the XML data.
#[derive(Debug, Clone)]
pub struct Variable {
    /// A unique identifier for the variable.
    pub id: String,
    /// The default value of the variable.
    pub default: String,
    /// The value attribute of the variable.
    value_attr: String,
    /// A list of dependencies for this variable.
    depends_array: Vec<Depend>,
    /// A list of references associated with this variable.
    ref_array: Vec<Reference>,
    /// An optional constraint applied to the variable.
    pub constraint: Option<Constraint>,
}

#[derive(Debug, Clone)]
pub struct Depend {
    re_f: String,
    _variables: Vec<Variable>,
}

impl Variables {
    fn new(variable_array: Vec<Variable>) -> Self {
        Variables { variable_array }
    }

    pub fn parse_variables<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Variables> {
        let mut variables: Vec<Variable> = Vec::new();

        let mut buf: Vec<u8> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    eprintln!("Error at position {}: {:?}", reader.error_position(), e);
                    return Err(XMLHandlerError::ParseError { source: e });
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"variable" => {
                    variables.push(Variable::parse_variable(reader, e.attributes())?);
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"variables" => {
                    return Ok(Variables::new(variables));
                }
                _ => (),
            }
        }
    }
}

impl Variable {
    fn new(
        id: String,
        default: String,
        value_attr: String,
        depends_array: Vec<Depend>,
        ref_array: Vec<Reference>,
        constraint: Option<Constraint>,
    ) -> Self {
        Variable {
            id,
            default,
            value_attr,
            depends_array,
            ref_array,
            constraint,
        }
    }

    pub fn parse_variable<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Variable> {
        let mut id = String::new();
        let mut default = String::new();
        let mut value_attr = String::new();

        let mut ref_array: Vec<Reference> = Vec::new();
        let mut depends_array: Vec<Depend> = Vec::new();
        let mut constraint: Option<Constraint> = None;

        let mut buf: Vec<u8> = Vec::new();

        for attr in attributes {
            let attr = attr?;
            match attr.key {
                QName(b"id") => id = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                QName(b"value") => {
                    value_attr = String::from_utf8_lossy(attr.value.as_ref()).to_string()
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
                Ok(Event::Start(e)) if e.name().as_ref() == b"default" => {
                    // Read text content of <default> tag
                    match reader.read_event_into(&mut buf) {
                        Err(e) => {
                            eprintln!("Error at position {}: {:?}", reader.error_position(), e);
                            return Err(XMLHandlerError::ParseError { source: e });
                        }
                        Ok(Event::Text(e)) => {
                            //default = e.unescape().unwrap().to_string();
                            match e.unescape() {
                                Ok(text) => default = text.to_string(),
                                Err(e) => {
                                    eprintln!("Error reading default value: {:?}", e);
                                    return Err(XMLHandlerError::ParseError { source: e });
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(e)) if e.name().as_ref() == b"reference" => {
                    ref_array.push(Reference::parse_reference_attr_only(e.attributes())?);
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"reference" => {
                    ref_array.push(Reference::parse_reference(reader, e.attributes())?);
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"constraints" => {
                    constraint = Some(Constraint::parse_constraint(reader, e.attributes())?);
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"depends" => {
                    depends_array.push(Depend::parse_depend(reader, e.attributes())?);
                }
                Ok(Event::End(e))
                    if e.name().as_ref() == b"case" || e.name().as_ref() == b"variable" =>
                {
                    return Ok(Variable::new(
                        id,
                        default,
                        value_attr,
                        depends_array,
                        ref_array,
                        constraint,
                    ));
                }
                _ => (),
            }
        }
    }
}

impl Depend {
    fn new(re_f: String, _variables: Vec<Variable>) -> Self {
        Depend { re_f, _variables }
    }

    fn parse_depend<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Depend> {
        let mut re_f = String::new();
        let mut _variables: Vec<Variable> = Vec::new();

        let mut buf: Vec<u8> = Vec::new();

        for attr in attributes {
            let attr = attr?;
            if let QName(b"ref") = attr.key {
                re_f = String::from_utf8_lossy(attr.value.as_ref()).to_string();
            }
        }

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    eprintln!("Error at position {}: {:?}", reader.error_position(), e);
                    return Err(XMLHandlerError::ParseError { source: e });
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"case" => {
                    _variables.push(Variable::parse_variable(reader, e.attributes())?);
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"depends" => {
                    return Ok(Depend::new(re_f, _variables));
                }
                _ => (),
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    // Non-testable helper function
    fn parse_variable_from_xml(xml: &str) -> Result<Variables> {
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut variables: Vec<Variable> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    eprintln!("Error at position {}: {:?}", reader.buffer_position(), e);
                    return Err(XMLHandlerError::ParseError { source: e });
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"variable" => {
                    variables.push(Variable::parse_variable(&mut reader, e.attributes())?);
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"variables" => {
                    return Ok(Variables::new(variables));
                }
                _ => (),
            }
        }
    }

    #[test]
    fn test_variable_with_error() {
        let invalid_xml = r#"<variables>
                                <variable id="var1">
                                    <default>&1</default>
                                    <constraints>
                                        <min>1</min>
                                        <max>100</max>
                                    </constraints>
                            </variable>
                        </variables>"#;

        let result = parse_variable_from_xml(invalid_xml);
        assert!(result.is_err(), "Expected an error, but got Ok");

        // Optionally, you can also check the error message
        if let Err(e) = result {
            assert_eq!(e.to_string(), "XML parsing error: Error while escaping character at range 0..2: Cannot find ';' after '&'");
        }
    }

    #[test]
    fn test_variable_with_valid_attributes() {
        let xml = r#"<variables>
                                <variable id="var1">
                                    <default>1</default>
                                    <constraints>
                                        <min>1</min>
                                        <max>100</max>
                                    </constraints>
                            </variable>
                        </variables>"#;

        match parse_variable_from_xml(xml) {
            Ok(vars) => {
                assert_eq!(vars.variable_array.len(), 1);
                assert_eq!(vars.variable_array[0].id, "var1");
                assert_eq!(vars.variable_array[0].default, "1");
                assert_eq!(vars.variable_array[0].value_attr, "");
                assert_eq!(vars.variable_array[0].depends_array.len(), 0);
                assert_eq!(vars.variable_array[0].ref_array.len(), 0);
                assert_eq!(vars.variable_array[0].constraint.is_some(), true);

                match vars.variable_array[0].constraint {
                    Some(ref c) => {
                        assert_eq!(c.min, 1.0);
                        assert_eq!(c.max, 100.0);
                    }
                    // If constraint is None, then fail the test
                    None => assert!(false),
                }
            }
            Err(e) => assert!(false, "Test failed due to error: {}", e),
        }
    }

    #[test]
    fn test_variable_with_depends_tag() {
        let xml = r#"<variables>
                                <variable id="var1">
                                    <depends ref="varFunction">
                                        <case value="case_1">
                                            <default>0</default>
                                            <reference id="ref_id_1"/>
                                        </case>
                                        <case value="case_2">
                                            <default>0</default>
                                            <reference id="ref_id_2"/>
                                        </case>
                                    </depends>
	                        </variable>
                        </variables>"#;

        match parse_variable_from_xml(xml) {
            Ok(vars) => {
                assert_eq!(vars.variable_array.len(), 1);
                assert_eq!(vars.variable_array[0].id, "var1");
                assert_eq!(vars.variable_array[0].default, "");
                assert_eq!(vars.variable_array[0].value_attr, "");
                assert_eq!(vars.variable_array[0].depends_array.len(), 1);
                assert_eq!(vars.variable_array[0].ref_array.len(), 0);
                assert_eq!(vars.variable_array[0].constraint.is_none(), true);

                let depend = &vars.variable_array[0].depends_array[0];
                assert_eq!(depend.re_f, "varFunction");
                assert_eq!(depend._variables.len(), 2);

                for (i, case) in depend._variables.iter().enumerate() {
                    assert_eq!(case.value_attr, format!("case_{}", i + 1));
                    assert_eq!(case.default, "0");
                    assert_eq!(case.depends_array.len(), 0);
                    assert_eq!(case.ref_array.len(), 1);
                    assert_eq!(case.constraint.is_none(), true);

                    let reference = &case.ref_array[0];
                    assert_eq!(reference.id, format!("ref_id_{}", i + 1));
                }
            }
            Err(e) => assert!(false, "Test failed due to error: {}", e),
        }
    }
}
