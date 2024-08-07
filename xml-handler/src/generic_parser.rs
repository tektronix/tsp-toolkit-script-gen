use quick_xml::events::Event;
use quick_xml::reader::Reader;

use std::path::Path;
use std::{
    fs::File,
    io::{Error, Read},
};

use crate::error::{Result, XMLHandlerError};
use crate::group_n_composite::Group;

pub fn parse_xml() -> Result<()> {
    let path = Path::new("xml-handler/src/resources/DefaultFunctionMetaData.xml");

    if let Ok(mut _file) = File::open(path) {
        //let mut file = File::open(path).unwrap();
        let mut buff = String::new();
        _file.read_to_string(&mut buff)?;

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
    } else {
        return Err(XMLHandlerError::IOError {
            source: Error::new(
                std::io::ErrorKind::NotFound,
                "Error: Could not locate file".to_string(),
            ),
        });
    }
}
