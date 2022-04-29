#![feature(if_let_guard)]

use std::any::Any;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct WTBLK {
	pub file_name: String,
	pub data: HashMap<String, WTType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WTType {
	Int(i64),
	Float(f64),
	String(String),
	Bool(bool),
	Array(Vec<WTType>),
	Struct(Box<WTBLK>),
}

impl From<&String> for WTType {
	fn from(input: &String) -> Self {
		return match () {
			_ if let Ok(int) = i64::from_str(input) => {
				Self::Int(int)
			}
			_ if let Ok(float) = f64::from_str(input) => {
				Self::Float(float)
			}
			_ if let Ok(bool) = bool::from_str(input) => {
				Self::Bool(bool)
			}
			_ => {
				Self::String(input.to_owned())
			}
		}
	}
}

pub enum BLKError {}

impl WTBLK {
	pub fn new_from_file(file: &str, file_name: &str) -> Result<Self, Box<dyn Any>> {
		let chars = file.chars();

		let mut data = HashMap::new();

		let mut escaping = false;
		let mut in_val = false;
		let mut buff = "".to_owned();
		let mut name = "".to_owned();
		let mut val = "".to_owned();

		for (i, char) in chars.enumerate() {
			match char {
				'"' => {
					if escaping {
						escaping = false;

						buff = buff.replace("\n", "").replace("\r\n", "");
						if in_val {
							val = buff.to_owned();
							data.insert(name.to_owned(), WTType::from(&val));
						} else {
							name = buff.to_owned();
						}
						buff.clear();
					} else {
						escaping = true;
					}
				}
				':' => {
					in_val = true;
				}
				',' => {
					in_val = false;
				}
				_ => {
					if in_val {
						val.push(char);
					} else {
						buff.push(char);
					}
				}
			}
		}

		Ok(Self {
			file_name: file_name.to_owned(),
			data,
		})
	}
}


#[cfg(test)]
mod tests {
	use std::fs;
	use crate::{WTBLK, WTType};

	#[test]
	fn test() {
		let wtblk = WTBLK::new_from_file(&fs::read_to_string("a_10a_late.blkx").unwrap(), "a_10a_late.blkx");
		eprintln!("{:#?}", wtblk.unwrap());
	}

	#[test]
	fn wt_value_from_int() {
		assert_eq!(WTType::from(&"5".to_string()), WTType::Int(5));
	}

	#[test]
	fn wt_value_from_float() {
		assert_eq!(WTType::from(&"5.5".to_string()), WTType::Float(5.5));
	}

	#[test]
	fn wt_value_from_bool() {
		assert_eq!(WTType::from(&"false".to_string()), WTType::Bool(false));
	}

	#[test]
	fn wt_value_from_string() {
		assert_eq!(WTType::from(&"text".to_string()), WTType::String("text".to_owned()));
	}
}
