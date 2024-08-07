use quick_xml::{events::Event, name::QName, Reader};
mod sub_mod;

pub use sub_mod::{Constraint, Reference};

#[derive(Debug)]
pub struct Variables {
    pub variable_array: Vec<Variable>,
}

#[derive(Debug)]
pub struct Variable {
    id: String,
    default: String,
    value_attr: String,
    depends_array: Vec<Depend>,
    ref_array: Vec<Reference>,
    constraint: Option<Constraint>,
}

#[derive(Debug)]
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
    ) -> quick_xml::Result<Variables> {
        let mut buf: Vec<u8> = Vec::new();

        let mut variables: Vec<Variable> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
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
    ) -> quick_xml::Result<Variable> {
        let mut buf: Vec<u8> = Vec::new();

        let mut id = String::new();
        let mut default = String::new();
        let mut value_attr = String::new();
        let mut ref_array: Vec<Reference> = Vec::new();
        let mut depends_array: Vec<Depend> = Vec::new();
        let mut constraint: Option<Constraint> = None;

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
                Ok(Event::Start(e)) if e.name().as_ref() == b"default" => {
                    // Read text content of <default> tag
                    match reader.read_event_into(&mut buf) {
                        Ok(Event::Text(e)) => {
                            default = e.unescape().unwrap().to_string();
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
    ) -> quick_xml::Result<Depend> {
        let mut buf: Vec<u8> = Vec::new();

        let mut re_f = String::new();
        let mut _variables: Vec<Variable> = Vec::new();

        for attr in attributes {
            let attr = attr?;
            match attr.key {
                QName(b"ref") => {
                    re_f = String::from_utf8_lossy(attr.value.as_ref()).to_string();
                }
                _ => {}
            }
        }

        loop {
            match reader.read_event_into(&mut buf) {
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
