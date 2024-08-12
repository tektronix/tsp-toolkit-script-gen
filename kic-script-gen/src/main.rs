use ::xml_handler::generic_parser;

fn main() -> anyhow::Result<()> {
    eprintln!("Welcome to TSP script generator");

    match generic_parser::parse_xml() {
        Ok(_) => eprintln!("Parsing successful"),
        Err(e) => {
            //eprintln!("Error: {:?}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
