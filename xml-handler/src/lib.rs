const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod generic_parser;

mod resources;

pub mod condition;
pub mod group_n_composite;
pub mod snippet;
pub mod substitute;
pub mod variable;

pub mod error;
