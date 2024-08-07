use crate::error::Result;
use quick_xml::{events::Event, name::QName, Reader};

#[derive(Debug)]
pub struct Substitute {
    name: String,
    value: String,
}

impl Substitute {
    /// Function to create a new instance of [`Substitute`]
    fn new(name: String, value: String) -> Self {
        Substitute { name, value }
    }

    pub fn parse_substitute<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        attributes: quick_xml::events::attributes::Attributes,
    ) -> Result<Substitute> {
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

        Ok(Substitute::new(name, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_substitute() {
        let substitute = Substitute::new(String::from("testName"), String::from("testValue"));
        assert_eq!(substitute.name, "testName");
        assert_eq!(substitute.value, "testValue");
    }

    #[test]
    fn test_parse_substitute() {
        let xml_data = r#"<substitute name="testName">%testValue%</substitute>"#;
        let mut reader = Reader::from_str(xml_data);

        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        // Move to the <substitute> start tag and collect its attributes
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name() == QName(b"substitute") => {
                    let result = Substitute::parse_substitute(&mut reader, e.attributes()).unwrap();

                    assert_eq!(result.name, "testName");
                    assert_eq!(result.value, "%testValue%");
                    break; // Assuming only one Substitute element for simplicity
                }
                Ok(Event::Eof) => break, // End of file
                _ => (), // There are other events like Text, End, etc., that we're not handling here
            }
            buf.clear();
        }
    }

    #[test]
    fn test_empty_substitute_tag() {
        let xml_data = r#"<substitute></substitute>"#;
        let mut reader = Reader::from_str(xml_data);

        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        // Move to the <substitute> start tag and collect its attributes
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name() == QName(b"substitute") => {
                    let result = Substitute::parse_substitute(&mut reader, e.attributes()).unwrap();

                    assert_eq!(result.name, "");
                    assert_eq!(result.value, "");
                    break; // Assuming only one Substitute element for simplicity
                }
                Ok(Event::Eof) => break, // End of file
                _ => (), // There are other events like Text, End, etc., that we're not handling here
            }
            buf.clear();
        }
    }

    #[test]
    fn test_invalid_xml_format() {
        let xml_data = r#"This is not XML"#;
        // Implement the test logic here

        let mut reader = Reader::from_str(xml_data);

        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        // Move to the <substitute> start tag and collect its attributes
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name() == QName(b"substitute") => {
                    let result = Substitute::parse_substitute(&mut reader, e.attributes()).unwrap();

                    assert_eq!(result.name, "");
                    assert_eq!(result.value, "");
                    break; // Assuming only one Substitute element for simplicity
                }
                Ok(Event::Eof) => break, // End of file
                _ => (), // There are other events like Text, End, etc., that we're not handling here
            }
            buf.clear();
        }
    }
}
