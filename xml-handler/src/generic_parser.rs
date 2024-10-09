use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::error::{Result, XMLHandlerError};
use crate::group::Group;
use crate::resources::DEFAULT_FUNC_METADATA;

/// Parses the XML data from the DefaultFunctionMetaData.xml file and returns a vector of `Group` objects.
///
/// # Errors
///
/// This function will return an `XMLHandlerError` if there is an error while reading
/// or parsing the XML data. The specific variant of `XMLHandlerError` will depend on the nature
/// of the error encountered.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(Vec<Group>)` containing a vector of `Group` objects if parsing is successful.
/// - `Err(XMLHandlerError)` if there is an error during parsing.
pub fn parse_xml() -> Result<Vec<Group>> {
    let binding = DEFAULT_FUNC_METADATA.to_string();
    let mut reader = Reader::from_str(binding.as_str());
    //reader.config_mut().trim_text(true);

    let mut buf: Vec<u8> = Vec::new();
    let mut groups: Vec<Group> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                eprintln!("Error at position {}: {:?}", reader.error_position(), e);
                return Err(XMLHandlerError::ParseError { source: e });
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

    //println!("{:#?}", groups);

    Ok(groups)
}
