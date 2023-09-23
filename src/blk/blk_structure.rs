use serde::{Deserialize, Serialize};

use crate::blk::blk_type::{BlkString, BlkType};
use crate::blk::util::blk_str;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlkField {
	// Name and field value
	Value(BlkString, BlkType),
	// Name and fields of substructs
	Struct(BlkString, Vec<BlkField>),
	// Array of merged fields that were duplicated in a Struct
	Merged(BlkString, Vec<BlkField>),
}

impl BlkField {
	pub fn new_root() -> Self {
		BlkField::Struct(blk_str("root"), vec![])
	}

	pub fn new_struct(name: BlkString) -> Self {
		BlkField::Struct(name, vec![])
	}

	#[must_use]
	pub fn insert_field(&mut self, field: Self) -> Option<()> {
		match self {
			BlkField::Struct(_, fields) => {
				fields.push(field);
				Some(())
			},
			_ => None,
		}
	}

	pub fn get_name(&self) -> String {
		match self {
			BlkField::Value(name, _) => name.to_string(),
			BlkField::Struct(name, _) => name.to_string(),
			BlkField::Merged(name, _) => name.to_string(),
		}
	}

	// TODO: Fully implement this
	/// A string formatted as such `struct_name_a/struct_name_c/field_name`
	/// Only takes relative path from current object
	/// If the current variant is not a struct, it will return an error `NoSuchField`
	pub fn pointer(&self, ptr: impl ToString) -> Result<Self, BlkFieldError> {
		let commands = ptr.to_string().split("/").map(|x| x.to_string()).collect();
		self.pointer_internal(commands, &mut 0_usize)
	}

	fn pointer_internal(
		&self,
		pointers: Vec<String>,
		at: &mut usize,
	) -> Result<Self, BlkFieldError> {
		let _current_search = pointers.get(*at);
		unimplemented!();
	}

	pub fn estimate_size(&self) -> usize {
		let mut total = 0;
		self._estimate_size(&mut total);
		total
	}

	fn _estimate_size(&self, total: &mut usize) {
		match self {
			BlkField::Value(key, value) => {
				*total += key.len();
				*total += value.size_bytes();
			},
			BlkField::Struct(key, fields) | BlkField::Merged(key, fields) => {
				*total += key.len();
				for field in fields {
					field._estimate_size(total);
				}
			},
		}
	}
}

pub enum BlkFieldError {
	// First is full pointer, 2nd is offending / missing string
	NoSuchField(String, String),
}
