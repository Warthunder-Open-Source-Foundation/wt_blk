use std::fmt::{Display, Formatter};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum WTBlkError<'a> {
	NoSuchValue(&'a str),

	Parse(&'a str),
	Serde(#[from] serde_json::Error)
}

impl Display for WTBlkError<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			WTBlkError::Serde(e) => {
				write!(f, "{}", e.to_string())
			}
			WTBlkError::NoSuchValue(pointer) => {
				write!(f, "No field for pointer {pointer}")
			}
			WTBlkError::Parse(pointer) => {
				write!(f, "Parsing the field for pointer {pointer} failed")
			}
		}
	}
}