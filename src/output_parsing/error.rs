use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WTBlkError {
    NoSuchValue(String),

    Parse(String, String),

    Serde(#[from] serde_json::Error),
}

impl Display for WTBlkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WTBlkError::Serde(e) => {
                write!(f, "{}", e.to_string())
            }
            WTBlkError::NoSuchValue(pointer) => {
                write!(f, "No field for pointer {pointer}")
            }
            WTBlkError::Parse(pointer, contents) => {
                write!(f, "{pointer} resolved correctly, but field failed to parse. contents: \n {contents}")
            }
        }
    }
}
