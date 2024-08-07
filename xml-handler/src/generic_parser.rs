use quick_xml::events::Event;
use quick_xml::reader::Reader;

use std::path::Path;
use std::{fs::File, io::Read};

use crate::group_n_composite::Group;

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
                    let group = Group::parse_group(&mut reader, e.attributes())?;
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
