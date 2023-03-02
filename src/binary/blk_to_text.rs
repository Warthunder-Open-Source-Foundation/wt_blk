use crate::binary::blk_structure::BlkField;

impl BlkField {
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
	use crate::binary::blk_structure::BlkField;
	use crate::binary::blk_type::BlkType;

	#[test]
	fn test_expected() {
		let mut root = BlkField::new_root();
		root.insert_field(BlkField::Value("vec4f".to_owned(), BlkType::Float4([1.25, 2.5, 5.0, 10.0]))).unwrap();
		root.insert_field(BlkField::Value("int".to_owned(), BlkType::Int(42))).unwrap();
		root.insert_field(BlkField::Value("long".to_owned(), BlkType::Long(42))).unwrap();

		let mut alpha = BlkField::new_struct("alpha");
		alpha.insert_field(BlkField::Value("str".to_owned(), BlkType::Str("hello".to_owned()))).unwrap();
		alpha.insert_field(BlkField::Value("bool".to_owned(), BlkType::Bool(true))).unwrap();
		alpha.insert_field(BlkField::Value("color".to_owned(), BlkType::Color([1, 2, 3, 4]))).unwrap();

		let mut gamma = BlkField::new_struct("gamma");
		gamma.insert_field(BlkField::Value("vec2i".to_owned(), BlkType::Int2([3, 4]))).unwrap();
		gamma.insert_field(BlkField::Value("vec2f".to_owned(), BlkType::Float2([1.25, 2.5]))).unwrap();
		gamma.insert_field(BlkField::Value("transform".to_owned(), BlkType::Float12(Box::new([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.25, 2.5, 5.0])))).unwrap();
		alpha.insert_field(gamma).unwrap();
		root.insert_field(alpha).unwrap();

		let mut beta = BlkField::new_struct("beta");
		beta.insert_field(BlkField::Value("float".to_owned(), BlkType::Float(1.25))).unwrap();
		beta.insert_field(BlkField::Value("vec2i".to_owned(), BlkType::Int2([1, 2]))).unwrap();
		beta.insert_field(BlkField::Value("vec3f".to_owned(), BlkType::Float3([1.25, 2.5, 5.0]))).unwrap();
		root.insert_field(beta).unwrap();


		println!("{}", root._as_blk_text(&mut 0, true));
	}
}