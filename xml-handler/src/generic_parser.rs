use quick_xml::reader::Reader;
use quick_xml::{events::Event, name::QName};

use std::path::Path;
use std::{fs::File, io::Read};

use crate::snippet::Snippet;
use crate::substitute::Substitute;
use crate::variable::{Variable, Variables};

#[derive(Debug)]
struct Group {
    id: String,
    type_: String,
    children: Vec<IncludeResult>,
    variable_List: Vec<Variable>,
}

#[derive(Debug)]
struct Composite {
    name: String,
    type_: Option<String>,

    substitutions: Vec<Substitute>,
    sub_children: Vec<IncludeResult>,
}

pub fn parse_xml() -> quick_xml::Result<()> {
    let path = Path::new("xml-handler/src/resources/DefaultFunctionMetaData.xml");
    let mut file = File::open(path).unwrap();
    let mut buff = String::new();
    file.read_to_string(&mut buff).unwrap();

    let mut reader = Reader::from_str(&buff);
    reader.config_mut().trim_text(true);

    let mut buf: Vec<u8> = Vec::new();
    let mut groups: Vec<Group> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error at position {}: {:?}", reader.error_position(), e)
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"group" => {
                    let group = parse_group(&mut reader, e.attributes())?;
                    groups.push(group);
                }
                _ => (),
            },

            _ => (),
        }
    }

    println!("{:#?}", groups);

    Ok(())
}

fn parse_group<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    attributes: quick_xml::events::attributes::Attributes,
) -> quick_xml::Result<Group> {
    let mut buf: Vec<u8> = Vec::new();
    let mut children: Vec<IncludeResult> = Vec::new();
    let mut variable_List: Vec<Variable> = Vec::new();

    let mut id = String::new();
    let mut type_ = String::new();

    for attr in attributes {
        let attr = attr?;
        match attr.key {
            QName(b"id") => id = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
            QName(b"type") => type_ = String::from_utf8(attr.value.into_owned()).unwrap(),
            _ => {}
        }
    }

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error at position {}: {:?}", reader.error_position(), e)
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"composite" => {
                let res = parse_composite(reader, e.attributes())?;
                children.push(IncludeResult::Composite(res));
            }
            Ok(Event::Empty(e)) if e.name().as_ref() == b"include" => {
                println!("inside include of group");
                let res = parse_include(e.attributes());
                match res {
                    Ok(ExternalFileResult::Snippet(snippet)) => {
                        //ToDo!
                    }
                    Ok(ExternalFileResult::Composite(composite)) => {
                        children.push(IncludeResult::Composite(composite));
                    }
                    Ok(ExternalFileResult::Variables(vars)) => {
                        variable_List = vars.variable_array
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"group" => {
                return Ok(Group {
                    id,
                    type_,
                    children,
                    variable_List,
                });
            }

            _ => (),
        }
    }
}

fn parse_composite<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    attributes: quick_xml::events::attributes::Attributes,
) -> quick_xml::Result<Composite> {
    let mut buf: Vec<u8> = Vec::new();
    let mut substitutions: Vec<Substitute> = Vec::new();
    let mut sub_children: Vec<IncludeResult> = Vec::new();

    let mut name = String::new();
    let mut type_: Option<String> = None;

    for attr in attributes {
        let attr = attr?;
        match attr.key {
            QName(b"name") => name = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
            QName(b"type") => type_ = Some(String::from_utf8(attr.value.into_owned()).unwrap()),
            _ => {}
        }
    }

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error at position {}: {:?}", reader.error_position(), e)
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
                    },
                }
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"snippet" => {
                let res = Snippet::parse_snippet(reader, e.attributes())?;
                sub_children.push(IncludeResult::Snippet(res));
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"composite" => {
                let res = parse_composite(reader, e.attributes())?;
                sub_children.push(IncludeResult::Composite(res));
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"composite" => {
                return Ok(Composite {
                    name,
                    type_,
                    substitutions,
                    sub_children,
                });
            }

            _ => (),
        }
    }
}

fn parse_include(
    attributes: quick_xml::events::attributes::Attributes,
) -> quick_xml::Result<ExternalFileResult> {
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
    println!("file_Path: {}", file_attr);

    let path = Path::new(file_attr.as_str());
    let mut file = File::open(path).unwrap();

    let mut buff = String::new();
    file.read_to_string(&mut buff).unwrap();

    let mut reader = Reader::from_str(&buff);
    reader.config_mut().trim_text(true);

    let mut buf: Vec<u8> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error at position {}: {:?}", reader.error_position(), e)
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"snippet" => {
                snippet = Some(Snippet::parse_snippet(&mut reader, e.attributes())?);
                return Ok(ExternalFileResult::Snippet(snippet.unwrap()));
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"composite" => {
                composite = Some(parse_composite(&mut reader, e.attributes())?);
                return Ok(ExternalFileResult::Composite(composite.unwrap()))
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"variables" => {
                let variables = Variables::parse_variables(&mut reader, e.attributes())?;
                return Ok(ExternalFileResult::Variables(variables));
            }
            _ => (),
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




