#![feature(if_let_guard)]

use std::str::FromStr;
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct WTBLK {
	pub struct_name: String,
	pub data: WTData,
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

pub type WTData = Vec<(String, WTType)>;

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
	pub fn new_from_file(file: &str, file_name: &str) -> Result<Self, Box<dyn Error>> {
		let file = file.replace("\r\n", "\n");


		let mut split = file.split("\n").collect::<Vec<&str>>();
		let mut data = Vec::new();
		let mut idx = 0;

		collect_inner_struct(&split, file_name, &mut data, &mut idx);

		Ok(Self {
			struct_name: file_name.to_owned(),
			data,
		})
	}
	pub fn new_from_type(file_name: &str, data: WTData) -> Self {
		Self {
			struct_name: file_name.to_owned(),
			data,
		}
	}
}

pub fn collect_inner_struct(file: &[&str], struct_name: &str, data: &mut WTData, idx: &mut usize) {
	let mut self_data: WTData = Vec::new();

	for row in file.iter().skip(*idx) {
		*idx += 1;
		println!("{idx} {}", row);
		let split = row.split(":").collect::<Vec<&str>>();

		if split.len() == 2 {
			let name = split[0];
			let val = split[1];
			if val.trim().contains("{") {
				println!("{idx} recurse into {}", name);
				collect_inner_struct(file, name.trim(), &mut self_data, idx);
			} else {
				println!("{idx} stow data as {name} with {val}");
				self_data.push((name.trim().to_owned(), WTType::from(&val.trim().replace(",", ""))));
			}
		} else {
			if row.contains("}") {
				println!("{idx} end recurse");
				data.push((struct_name.to_owned(), WTType::Struct(Box::new(WTBLK::new_from_type(struct_name, self_data)))));
				return;
			} else {
				if row.contains("{") {
					println!("{idx} recurse into unit_type");
					collect_inner_struct(file, "", &mut self_data, idx);
				}
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
