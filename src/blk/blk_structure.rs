use std::collections::HashMap;
use std::mem;
use std::mem::replace;
use indexmap::IndexMap;
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

	pub fn apply_overrides(&mut self) {
		match self {
			BlkField::Struct(_, values) => {
				// Move values out of struct, we will return it later
				let mut moved_values = mem::replace(values, vec![]);

				moved_values.iter_mut().for_each(|v| v.apply_overrides());

				// Left are overrides
				let with_name: (Vec<_>, Vec<_>) = moved_values.into_iter()
					.map(|e| (e.get_name(), e))
					.partition(|(name, _)| name.starts_with("override:"));


				// Map of to-replace keys
				let mut map: IndexMap<BlkString, BlkField> = IndexMap::from_iter(with_name.1);

				// Replace all keys where
				for (key, mut value) in with_name.0 {
					let replaced = key.replace("override:", "");
					if let Some(inner) = map.get_mut(replaced.as_str()) {
						value.set_name(blk_str(replaced.as_str()));
						*inner = value;
					}
				}
				*values = map.into_iter().map(|e|e.1).collect();
			}
			_ => {}
		}
	}

	#[must_use]
	pub fn insert_field(&mut self, field: Self) -> Option<()> {
		match self {
			BlkField::Struct(_, fields) => {
				fields.push(field);
				Some(())
			}
			_ => None,
		}
	}

	pub fn get_name(&self) -> BlkString {
		match self {
			BlkField::Value(name, _) |
			BlkField::Struct(name, _) |
			BlkField::Merged(name, _) => {
				name.clone()
			}
		}
	}

	pub fn set_name(&mut self, new: BlkString) {
		match self {
			BlkField::Value(name, _) |
			BlkField::Struct(name, _) |
			BlkField::Merged(name, _) => {
				*name = new;
			}
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
			}
			BlkField::Struct(key, fields) | BlkField::Merged(key, fields) => {
				*total += key.len();
				for field in fields {
					field._estimate_size(total);
				}
			}
		}
	}
}

pub enum BlkFieldError {
	// First is full pointer, 2nd is offending / missing string
	NoSuchField(String, String),
}

#[cfg(test)]
mod test {
	use crate::blk::blk_structure::BlkField;
	use crate::blk::blk_type::BlkType;
	use crate::blk::util::blk_str;

	#[test]
	fn should_override() {
		let mut before = BlkField::new_root();
		before.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(0))).unwrap();
		before.insert_field(BlkField::Value(blk_str("override:value"), BlkType::Int(42))).unwrap();
		before.apply_overrides();

		let mut expected = BlkField::new_root();
		expected.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(42))).unwrap();

		assert_eq!(before, expected);
	}

	#[test]
	fn preserve_order() {
		let mut after = BlkField::new_root();
		after.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(0))).unwrap();
		after.insert_field(BlkField::Value(blk_str("value3"), BlkType::Int(42))).unwrap();
		after.insert_field(BlkField::Value(blk_str("value71q234"), BlkType::Int(213123))).unwrap();
		let before = after.clone();
		after.apply_overrides();

		assert_eq!(after, before);
	}
}
