use crate::binary::blk_type::BlkType;


pub enum BlkField {
	// Name and field value
	Value(String, BlkType),
	// Name and fields of substructs
	Struct(String, Vec<BlkField>),
}


impl BlkField {
	pub fn new() -> Self {
		BlkField::Struct("root".to_owned(), vec![])
	}

	// TODO: Fully implement this
	/// A string formatted as such `struct_name_a/struct_name_c/field_name`
	/// Only takes relative path from current object
	/// If the current variant is not a struct, it will return an error `NoSuchField`
	/*
	pub fn pointer(&self, ptr: impl ToString) -> Result<Self, BlkFieldError> {
		let commands = ptr.to_string().split("/");
		self.pointer_internal(commands.collect(), &mut 0_usize)
	}

	fn pointer_internal(&self, pointers: Vec<String>, at: &mut usize) -> Result<Self, BlkFieldError> {
		let current_search = pointers.get(*at);
	}
	 */
}

pub enum BlkFieldError {
	// First is full pointer, 2nd is offending / missing string
	NoSuchField(String, String)
}