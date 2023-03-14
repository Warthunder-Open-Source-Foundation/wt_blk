use serde::{Deserialize, Serialize};
use serde_json::to_string;
use crate::binary::blk_type::{BlkCow, BlkType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlkField<'a> {
	// Name and field value
	Value(BlkCow<'a>, BlkType<'a>),
	// Name and fields of substructs
	Struct(BlkCow<'a>, Vec<BlkField<'a>>),
}


impl <'a>BlkField<'a> {
	pub fn new_root() -> Self {
		BlkField::Struct(BlkCow::Borrowed("root"), vec![])
	}

	pub fn new_struct(name: BlkCow<'a>) -> Self {
		BlkField::Struct(name, vec![])
	}

	#[must_use]
	pub fn insert_field(&mut self, field: Self) -> Option<()> {
		match self {
			BlkField::Value(_, _) => {
				None
			}
			BlkField::Struct(_, fields) => {
				fields.push(field);
				Some(())
			}
		}
	}

	pub fn get_name(&self) -> String {
		match self {
			BlkField::Value(name, _) => {name.to_string()}
			BlkField::Struct(name, _) => {name.to_string()}
		}
	}

	// TODO: Fully implement this
	/// A string formatted as such `struct_name_a/struct_name_c/field_name`
	/// Only takes relative path from current object
	/// If the current variant is not a struct, it will return an error `NoSuchField`
	pub fn pointer(&self, ptr: impl ToString) -> Result<Self, BlkFieldError> {
		let commands = ptr.to_string().split("/").map(|x|x.to_string()).collect();
		self.pointer_internal(commands, &mut 0_usize)
	}

	fn pointer_internal(&self, pointers: Vec<String>, at: &mut usize) -> Result<Self, BlkFieldError> {
		let current_search = pointers.get(*at);
		unimplemented!();
	}
}

pub enum BlkFieldError {
	// First is full pointer, 2nd is offending / missing string
	NoSuchField(String, String)
}