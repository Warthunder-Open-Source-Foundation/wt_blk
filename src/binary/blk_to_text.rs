use crate::binary::blk_structure::BlkField;

impl BlkField<'_> {
	// Public facing formatting fn
	pub fn as_blk_text(&self) -> String {
		self._as_blk_text(&mut 0, true)
	}

	// TODO: Make this generic with a configuration file
	// Internal fn that actually formats
	fn _as_blk_text(&self, indent_level: &mut usize, is_root: bool) -> String {
		match self {
			BlkField::Value(name, value) => {
				format!("\"{name}\":{value}")
			}
			BlkField::Struct(name, fields) => {
				let indent = "\t".repeat(*indent_level);
				*indent_level += 1;
				let children = fields.iter().map(|x| format!("{indent}{}", x._as_blk_text(indent_level, false))).collect::<Vec<_>>().join("\n");
				*indent_level -= 1;

				let indent_closing = "\t".repeat(indent_level.saturating_sub(1));
				if is_root {
					format!("{children}")
				} else {
					format!("\"{name}\" {{\n{children}\n{indent_closing}}}")

				}
			}
		}
	}
}

#[cfg(test)]
mod test {
	use std::borrow::Cow;
	use crate::binary::blk_structure::BlkField;
	use crate::binary::blk_type::BlkType;

	#[test]
	fn test_expected() {
		let mut root = BlkField::new_root();
		root.insert_field(BlkField::Value(Cow::Borrowed("vec4f"), BlkType::Float4([1.25, 2.5, 5.0, 10.0]))).unwrap();
		root.insert_field(BlkField::Value(Cow::Borrowed("int"), BlkType::Int(42))).unwrap();
		root.insert_field(BlkField::Value(Cow::Borrowed("long"), BlkType::Long(42))).unwrap();

		let mut alpha = BlkField::new_struct(Cow::Borrowed("alpha"));
		alpha.insert_field(BlkField::Value(Cow::Borrowed("str"), BlkType::Str(Cow::Borrowed("hello")))).unwrap();
		alpha.insert_field(BlkField::Value(Cow::Borrowed("bool"), BlkType::Bool(true))).unwrap();
		alpha.insert_field(BlkField::Value(Cow::Borrowed("color"), BlkType::Color([1, 2, 3, 4]))).unwrap();

		let mut gamma = BlkField::new_struct(Cow::Borrowed("gamma"));
		gamma.insert_field(BlkField::Value(Cow::Borrowed("vec2i"), BlkType::Int2([3, 4]))).unwrap();
		gamma.insert_field(BlkField::Value(Cow::Borrowed("vec2f"), BlkType::Float2([1.25, 2.5]))).unwrap();
		gamma.insert_field(BlkField::Value(Cow::Borrowed("transform"), BlkType::Float12(Box::new([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.25, 2.5, 5.0])))).unwrap();
		alpha.insert_field(gamma).unwrap();
		root.insert_field(alpha).unwrap();

		let mut beta = BlkField::new_struct(Cow::Borrowed("beta"));
		beta.insert_field(BlkField::Value(Cow::Borrowed("float"), BlkType::Float(1.25))).unwrap();
		beta.insert_field(BlkField::Value(Cow::Borrowed("vec2i"), BlkType::Int2([1, 2]))).unwrap();
		beta.insert_field(BlkField::Value(Cow::Borrowed("vec3f"), BlkType::Float3([1.25, 2.5, 5.0]))).unwrap();
		root.insert_field(beta).unwrap();


		println!("{}", root._as_blk_text(&mut 0, true));
	}
}