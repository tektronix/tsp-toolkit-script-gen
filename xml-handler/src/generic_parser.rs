use quick_xml::reader::Reader;
use quick_xml::{events::Event, name::QName};

use std::{fs::File, io::Read};

#[derive(Debug)]
struct Group {
    id: String,
    type_: String,
    composites: Vec<Composite>,
}

#[derive(Debug)]
struct Composite {
    name: String,
    type_: Option<String>,
    substitutes: Vec<Substitute>,
    includes: Vec<Composite>, // Changed to store included composites directly
}

#[derive(Debug)]
struct Substitute {
    name: String,
    value: String,
}

pub fn parse_xml() -> quick_xml::Result<()> {
    let mut file = File::open("xml-handler\\src\\resources\\DefaultFunctionMetaData.xml").unwrap();
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
    let mut composites: Vec<Composite> = Vec::new();

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
                composites.push(parse_composite(reader, e.attributes())?);
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"group" => {
                return Ok(Group {
                    id,
                    type_,
                    composites,
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
    let mut substitutes: Vec<Substitute> = Vec::new();
    let mut includes: Vec<Composite> = Vec::new();

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
                substitutes.push(parse_substitute(reader, e.attributes())?);
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"include" => {
                //substitutes.push(parse_include(reader, e.attributes())?);
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"composite" => {
                return Ok(Composite {
                    name,
                    type_,
                    substitutes,
                    includes,
                });
            }

            _ => (),
        }
    }
}

fn parse_substitute<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    attributes: quick_xml::events::attributes::Attributes,
) -> quick_xml::Result<Substitute> {
    let mut name = String::new();
    let mut value = String::new();
    let mut buf: Vec<u8> = Vec::new();

    for attr in attributes {
        let attr = attr?;
        match attr.key {
            QName(b"name") => name = String::from_utf8_lossy(attr.value.as_ref()).to_string(),
            //QName(b"value") => value = String::from_utf8(attr.value.into_owned()).unwrap(),
            _ => {}
        }
    }

    match reader.read_event_into(&mut buf) {
        Ok(Event::Text(e)) => {
            // Capture the text content inside the <substitute> tag
            match e.unescape() {
                Ok(text) => value = text.to_string(),
                Err(_) => {}
            }
        }
        _ => (),
    }

    Ok(Substitute { name, value })
}

fn parse_include<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    attributes: quick_xml::events::attributes::Attributes,
) {
    //
}
