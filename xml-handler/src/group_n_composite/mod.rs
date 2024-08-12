use quick_xml::{events::Event, name::QName, Reader};

use crate::resources::Resource;
use crate::{
    error::{Result, XMLHandlerError},
    snippet::Snippet,
    substitute::Substitute,
    variable::{Variable, Variables},
};

#[derive(Debug)]
pub struct Group {
    id: String,
    type_: String,
    children: Vec<IncludeResult>,
    variable_list: Vec<Variable>,
}

impl Group {
    fn new(
        id: String,
        type_: String,
        children: Vec<IncludeResult>,
        variable_list: Vec<Variable>,
    ) -> Self {
        Group {
            id,
            type_,
            children,
            variable_list,
        }
    }

    pub fn parse_group<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Group> {
        let mut id = String::new();
        let mut type_ = String::new();

        let mut children: Vec<IncludeResult> = Vec::new();
        let mut variable_list: Vec<Variable> = Vec::new();

        let mut buf: Vec<u8> = Vec::new();

        for attr in attributes {
            let attr = attr?;
            match attr.key {
                QName(b"id") => id = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                QName(b"type") => type_ = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                _ => {}
            }
        }

        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => {
                    eprintln!("Error at position {}: {:?}", reader.error_position(), e);
                    return Err(XMLHandlerError::ParseError { source: e });
                }
                Ok(Event::Start(e)) if e.name().as_ref() == b"composite" => {
                    let res = Composite::parse_composite(reader, e.attributes())?;
                    children.push(IncludeResult::Composite(res));
                }
                Ok(Event::Empty(e)) if e.name().as_ref() == b"include" => {
                    let res = parse_include(e.attributes());
                    match res {
                        Ok(ExternalFileResult::Snippet(snippet)) => {
                            //ToDo!
                        }
                        Ok(ExternalFileResult::Composite(composite)) => {
                            children.push(IncludeResult::Composite(composite));
                        }
                        Ok(ExternalFileResult::Variables(vars)) => {
                            variable_list = vars.variable_array
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(e)) if e.name().as_ref() == b"group" => {
                    return Ok(Group::new(id, type_, children, variable_list));
                }

                _ => {}
            }
        }
    }
}

#[derive(Debug)]
struct Composite {
    name: String,
    type_: Option<String>,

    substitutions: Vec<Substitute>,
    sub_children: Vec<IncludeResult>,
}

impl Composite {
    fn new(
        name: String,
        type_: Option<String>,
        substitutions: Vec<Substitute>,
        sub_children: Vec<IncludeResult>,
    ) -> Self {
        Composite {
            name,
            type_,
            substitutions,
            sub_children,
        }
    }

    fn parse_composite<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Composite> {
        let mut name = String::new();
        let mut type_: Option<String> = None;

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
                            //ToDo!
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
                    return Ok(Composite::new(name, type_, substitutions, sub_children));
                }

                _ => (),
            }
        }
    }
}

fn parse_include(
    attributes: quick_xml::events::attributes::Attributes,
) -> Result<ExternalFileResult> {
    let mut file_attr = String::new();
    let mut snippet: Option<Snippet> = None;
    let mut composite: Option<Composite> = None;

    for attr in attributes {
        let attr = attr?;
        match attr.key {
            QName(b"path") => file_attr = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
            _ => {}
        }
    }

    match Resource::match_resource(file_attr.as_str()) {
        Some(res) => {
            let xml_string = res.to_string();
            let mut reader = Reader::from_str(xml_string.as_str());
            //reader.config_mut().trim_text(true);

            let mut buf: Vec<u8> = Vec::new();

            loop {
                match reader.read_event_into(&mut buf) {
                    Err(e) => {
                        eprintln!("Error at position {}: {:?}", reader.error_position(), e);
                        return Err(XMLHandlerError::ParseError { source: e });
                    }
                    Ok(Event::Start(e)) if e.name().as_ref() == b"snippet" => {
                        snippet = Some(Snippet::parse_snippet(&mut reader, e.attributes())?);
                        return Ok(ExternalFileResult::Snippet(snippet.unwrap()));
                    }
                    Ok(Event::Start(e)) if e.name().as_ref() == b"composite" => {
                        composite = Some(Composite::parse_composite(&mut reader, e.attributes())?);
                        return Ok(ExternalFileResult::Composite(composite.unwrap()));
                    }
                    Ok(Event::Start(e)) if e.name().as_ref() == b"variables" => {
                        let variables = Variables::parse_variables(&mut reader, e.attributes())?;
                        return Ok(ExternalFileResult::Variables(variables));
                    }
                    _ => (),
                }
            }
        }
        None => {
            //TODO: Handle error
            return Err(XMLHandlerError::UnknownXMLFileError {
                file_name: file_attr,
            });
        }
    }
}

#[derive(Debug)]
enum ExternalFileResult {
    Snippet(Snippet),
    Composite(Composite),
    Variables(Variables),
}

#[derive(Debug)]
enum IncludeResult {
    Snippet(Snippet),
    Composite(Composite),
}

#[cfg(test)]
mod tests {
    use super::*;

    // Non-testable helper function
    fn parse_groups_from_xml(xml: &str) -> Result<Vec<Group>> {
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut groups: Vec<Group> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"group" => match Group::parse_group(&mut reader, e.attributes()) {
                        Ok(group) => groups.push(group),
                        Err(e) => eprintln!("Error parsing group: {}", e),
                    },
                    _ => (),
                },
                Ok(Event::Eof) => break,
                Err(e) => {
                    eprintln!("Error reading XML: {}", e);
                    break;
                }
                _ => (),
            }
            buf.clear();
        }

        Ok(groups)
    }

    #[test]
    fn test_parse_group_with_valid_attributes() {
        let xml = r#"<group id="test_group" type="example_type"></group>"#;

        match parse_groups_from_xml(xml) {
            Ok(groups) => {
                assert_eq!(groups[0].id, "test_group");
                assert_eq!(groups[0].type_, "example_type");
            }
            Err(e) => eprintln!("Error parsing group: {}", e),
        }
    }

    #[test]
    fn test_parse_group_with_missing_attributes() {
        let xml = r#"<group id="test_group"></group>"#;

        match parse_groups_from_xml(xml) {
            Ok(groups) => {
                assert_eq!(groups[0].id, "test_group");
                assert_eq!(groups[0].type_, "");
            }
            Err(e) => eprintln!("Error parsing group: {}", e),
        }
    }

    #[test]
    fn test_parse_composite_with_valid_attributes() {
        let xml = r#"<group id="test_group" type="example_type">
                            <composite name="test_composite" type="example_type">
                            </composite>
                        </group>"#;

        match parse_groups_from_xml(xml) {
            Ok(groups) => {
                println!("{:?}", groups);
                assert_eq!(groups[0].children.len(), 1);
                if let IncludeResult::Composite(composite) = &groups[0].children[0] {
                    assert_eq!(composite.name, "test_composite");
                    assert_eq!(composite.type_.as_deref(), Some("example_type"));
                } else {
                    // Fail the test if the first child is not a composite
                    assert!(false);
                }
            }
            Err(e) => eprintln!("Error parsing group: {}", e),
        }
    }

    #[test]
    fn test_parse_composite_with_substitutions() {
        let xml = r#"<group id="test_group" type="example_type"> 
                                <composite name="test_composite" type="example_type">
                                    <substitute name="test_substitute1">test_value1</substitute>
                                    <substitute name="test_substitute2">test_value2</substitute>
                                </composite>
                        </group>"#;

        match parse_groups_from_xml(xml) {
            Ok(groups) => {
                println!("{:?}", groups);
                assert_eq!(groups[0].children.len(), 1);
                if let IncludeResult::Composite(composite) = &groups[0].children[0] {
                    assert_eq!(composite.name, "test_composite");
                    assert_eq!(composite.type_.as_deref(), Some("example_type"));
                    assert_eq!(composite.substitutions.len(), 2);
                    for (i, sub) in composite.substitutions.iter().enumerate() {
                        assert_eq!(sub.name, format!("test_substitute{}", i + 1));
                        assert_eq!(sub.value, format!("test_value{}", i + 1));
                    }
                } else {
                    // Fail the test if the first child is not a composite
                    assert!(false);
                }
            }
            Err(e) => eprintln!("Error parsing group: {}", e),
        }
    }

    #[test]
    fn test_parse_composite_with_snippet() {
        let xml = r#"<group id="test_group" type="example_type">
                                <composite>
                                        <snippet>
                                            <condition name="CONDITION_1">VALUE_1</condition>
                                            sample code for snippet - 1
                                        </snippet>
                                        <snippet>
                                            <condition name="CONDITION_2">VALUE_2</condition>
                                            sample code for snippet - 2
                                        </snippet>
                                </composite>
                        </group>"#;

        match parse_groups_from_xml(xml) {
            Ok(groups) => {
                println!("{:?}", groups);
                assert_eq!(groups[0].children.len(), 1);
                if let IncludeResult::Composite(composite) = &groups[0].children[0] {
                    assert_eq!(composite.name, "");
                    assert_eq!(composite.type_.as_deref(), None);
                    assert_eq!(composite.sub_children.len(), 2);

                    for (i, child) in composite.sub_children.iter().enumerate() {
                        if let IncludeResult::Snippet(snippet) = child {
                            assert_eq!(snippet.name, "");
                            assert_eq!(snippet.repeat, "");
                            //TODO: Check if indentation causes any issue during actual value substitution and script generation
                            assert!(snippet
                                .code_snippet
                                .contains(&format!("sample code for snippet - {}", i + 1)));
                            assert_eq!(snippet.conditions.len(), 1);
                            assert_eq!(snippet.conditions[0].name, format!("CONDITION_{}", i + 1));
                            assert_eq!(snippet.conditions[0].value, format!("VALUE_{}", i + 1));
                        } else {
                            // Fail the test if the first child is not a snippet
                            assert!(false);
                        }
                    }
                } else {
                    // Fail the test if the first child is not a composite
                    assert!(false);
                }
            }
            Err(e) => eprintln!("Error parsing group: {}", e),
        }
    }
}
