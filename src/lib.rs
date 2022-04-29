#![feature(if_let_guard)]

use std::any::Any;
use std::collections::HashMap;
use std::str::FromStr;
use std::str::Chars;

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
		};
	}
}

pub enum BLKError {}

impl WTBLK {
	pub fn new_from_file(file: &str, file_name: &str) -> Result<Self, Box<dyn Any>> {
		let file = file.replace("\r\n", "\n");

		let mut data = HashMap::new();
		let mut idx = 0;

		collect_inner_struct(&file, file_name, &mut data, &mut idx);

		Ok(Self {
			file_name: file_name.to_owned(),
			data,
		})
	}
	pub fn new_from_type(file_name: &str, data: HashMap<String, WTType>) -> Self {
		Self {
			file_name: file_name.to_owned(),
			data
		}
	}
}

pub fn collect_inner_struct(file: &str, file_name: &str, data: &mut HashMap<String, WTType>, idx: &mut usize) {
	let mut escaping = false;
	let mut in_val = false;
	let mut buff = "".to_owned();
	let mut name = "".to_owned();
	let mut val;

	let mut self_data = HashMap::new();


	for char in file.split_at(*idx).1.chars() {
		*idx += 1;
		match char {
			'"' => {
				if escaping {
					escaping = false;

					if !in_val {
						name = buff.to_owned();
						buff.clear();
					}
				} else {
					escaping = true;
				}
			}
			':' => {
				in_val = true;
			}
			'}' => {
				data.insert(name.to_owned(), WTType::Struct(Box::new(WTBLK::new_from_type(file_name, self_data))));
				return;
			}
			'{' => {
				collect_inner_struct(file, &name, data, idx);
			}
			'\n' => {
				val = buff.trim().replace(",", "").to_owned();
				self_data.insert(name.trim().to_owned(), WTType::from(&val));
				buff.clear();

				in_val = false;
			}
			_ => {
				buff.push(char);
			}
		}
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
