use chrono::Local;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StatusMsg {
    pub status_type: StatusType,
    pub message: String,
    pub time_stamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum StatusType {
    Info,
    Warning,
    Error,
}

impl StatusMsg {
    #[must_use]
    pub fn new(status_type: StatusType, message: String) -> Self {
        let time_stamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            status_type,
            message,
            time_stamp,
        }
    }
}
