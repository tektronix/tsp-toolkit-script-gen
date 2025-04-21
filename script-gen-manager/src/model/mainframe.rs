use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mainframe {
    pub name: String,
    pub model: String,
    pub slot: Vec<Slot>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Slot {
    pub name: String,
    pub model: String,
}
