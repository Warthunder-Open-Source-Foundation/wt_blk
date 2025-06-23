use std::{fmt::Debug, iter::Peekable, mem};
use color_eyre::eyre::bail;
use color_eyre::Report;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::blk::blk_type::BlkType;
use crate::blk::blk_string::{blk_str, BlkString};

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

	pub fn apply_overrides(&mut self, already_merged_fields: bool) {
		match self {
			BlkField::Struct(_, values) => {
				// Move values out of struct, we will return it later
				let mut moved_values = mem::replace(values, vec![]);

				moved_values.iter_mut().for_each(|v| v.apply_overrides(already_merged_fields));

				// Non-overriding fastpath
				if !moved_values.iter().any(|e|e.get_name().starts_with("override:")) {
					*values = moved_values;
					return;
				}

				// Left are overrides
				let with_name: (Vec<_>, Vec<_>) = moved_values
					.into_iter()
					.map(|e| (e.get_name(), e))
					.partition(|(name, _)| name.starts_with("override:"));


				// Unmerged fields cannot use hashmap to find overrides, a linear search is required instead
				let map: &mut dyn Iterator<Item = _> = if already_merged_fields {
					// Map of to-replace keys
					let mut map: IndexMap<BlkString, BlkField> = IndexMap::from_iter(with_name.1);
					for (key, mut value) in with_name.0 {
						let replaced = BlkString::from(key.replace("override:", ""));
						if let Some(inner) = map.get_mut(&replaced) {
							value.set_name(blk_str(replaced.as_str()));
							*inner = value;
						}
					}
					&mut map.into_iter()
				} else {
					let mut map = with_name.1;
					for (key, mut value) in with_name.0 {
						let replaced = key.replace("override:", "");

						for (_, inner) in map.iter_mut() {
							if inner.get_name().as_ref() == replaced {
								value.set_name(blk_str(replaced.as_str()));
								*inner = value.clone();
							}
						}
					}
					&mut map.into_iter()
				};


				*values = map.into_iter().map(|e| e.1).collect();
			},
			_ => {},
		}
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

	pub fn get_name(&self) -> BlkString {
		match self {
			BlkField::Value(name, _) | BlkField::Struct(name, _) | BlkField::Merged(name, _) => {
				name.clone()
			},
		}
	}

	pub fn set_name(&mut self, new: BlkString) {
		match self {
			BlkField::Value(name, _) | BlkField::Struct(name, _) | BlkField::Merged(name, _) => {
				*name = new;
			},
		}
	}

	pub fn value(&self) -> Option<&BlkType> {
		match self {
			BlkField::Value(_, v)  => {
				Some(v)
			},
			_ => {panic!("Field is not a value")}
		}
	}

	pub fn pointer(&self, ptr: &str) -> Result<BlkField, Report> {
		let commands = ptr.split("/");
		self.pointer_internal(&mut commands.into_iter().peekable())
	}

	fn pointer_internal<'a>(
		&self,
		pointers: &mut Peekable<impl Iterator<Item = &'a str>>,
	) -> Result<BlkField, Report> {
		let current_search = pointers.next();
		match self {
			BlkField::Value(_k, _v) => {
				if let Some(needle) = current_search {
					bail!("{needle} is a value, not a struct")
				} else {
					Ok(self.clone())
				}
			},
			BlkField::Struct(k, v) | BlkField::Merged(k, v) => {
				if let Some(search) = current_search {
					for value in v {
						if value.get_name().as_str() == search {
							return value.pointer_internal(pointers);
						}
					}
					bail!("{search} is not contained in {k}")
				} else {
					bail!("key is not contained in {}", self.get_name())
				}
			},
		}
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

#[cfg(test)]
mod test {
	use crate::blk::{blk_structure::BlkField, blk_type::BlkType, make_strict_test};
	use crate::blk::blk_string::blk_str;

	#[test]
	fn should_override() {
		let mut before = BlkField::new_root();
		before
			.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(0)))
			.unwrap();
		before
			.insert_field(BlkField::Value(blk_str("override:value"), BlkType::Int(42)))
			.unwrap();
		before.apply_overrides(true);

		let mut expected = BlkField::new_root();
		expected
			.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(42)))
			.unwrap();

		assert_eq!(before, expected);
	}

	#[test]
	fn preserve_order() {
		let mut after = BlkField::new_root();
		after
			.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(0)))
			.unwrap();
		after
			.insert_field(BlkField::Value(blk_str("value3"), BlkType::Int(42)))
			.unwrap();
		after
			.insert_field(BlkField::Value(
				blk_str("value71q234"),
				BlkType::Int(213123),
			))
			.unwrap();
		let before = after.clone();
		after.apply_overrides(true);

		assert_eq!(after, before);
	}

	#[test]
	fn keep_dupe_fields() {
		let mut after = BlkField::new_root();
		after
			.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(0)))
			.unwrap();
		after
			.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(42)))
			.unwrap();
		after
			.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(123)))
			.unwrap();
		let before = after.clone();
		after.apply_overrides(false);

		assert_eq!(after, before);
	}

	#[test]
	fn keep_dupe_fields_override() {
		let mut before = BlkField::new_root();
		before
			.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(0)))
			.unwrap();
		before
			.insert_field(BlkField::Value(blk_str("override:cheese"), BlkType::Int(69)))
			.unwrap();
		before
			.insert_field(BlkField::Value(blk_str("cheese"), BlkType::Int(690)))
			.unwrap();
		before
			.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(42)))
			.unwrap();
		before
			.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(123)))
			.unwrap();
		let mut expected = BlkField::new_root();

		expected
			.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(0)))
			.unwrap();
		expected
			.insert_field(BlkField::Value(blk_str("cheese"), BlkType::Int(69)))
			.unwrap();
		expected
			.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(42)))
			.unwrap();
		expected
			.insert_field(BlkField::Value(blk_str("value"), BlkType::Int(123)))
			.unwrap();

		before.apply_overrides(false);

		assert_eq!(before, expected);
	}

	#[test]
	fn ptr_value() {
		let blk = make_strict_test();
		assert_eq!(*blk.pointer("int").unwrap().value().unwrap(), BlkType::Int(42));
		assert_eq!(*blk.pointer("alpha/str").unwrap().value().unwrap(), BlkType::Str(blk_str("hello")));
		assert!(blk.pointer("alpha/noexist").is_err());
	}
}
