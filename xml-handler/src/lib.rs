const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod generic_parser;

mod resources;

pub mod composite;
pub mod condition;
pub mod group;
pub mod snippet;
pub mod substitute;
pub mod variable;

pub mod error;
