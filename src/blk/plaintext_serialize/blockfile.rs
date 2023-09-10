use color_eyre::{eyre::bail, Report};

use crate::blk::blk_structure::BlkField;

impl BlkField {
	// Public facing formatting fn
	pub fn as_blk_text(&self) -> Result<String, Report> {
		self.inner_as_blk_text(&mut 0, true)
	}

	// TODO: Make this generic with a configuration file
	// Internal fn that actually formats
	fn inner_as_blk_text(&self, indent_level: &mut usize, is_root: bool) -> Result<String, Report> {
		match self {
			BlkField::Value(name, value) => Ok(format!("\"{name}\":{value}")),
			BlkField::Struct(name, fields) => {
				let indent = "\t".repeat(*indent_level);
				*indent_level += 1;
				let children = fields
					.iter()
					.map(|x| {
						Ok(format!(
							"{indent}{}",
							x.inner_as_blk_text(indent_level, false)?
						))
					})
					.collect::<Result<Vec<_>, Report>>()?
					.join("\n");
				*indent_level -= 1;

				let indent_closing = "\t".repeat(indent_level.saturating_sub(1));
				Ok(if is_root {
					format!("{children}")
				} else {
					format!("\"{name}\" {{\n{children}\n{indent_closing}}}")
				})
			},
			BlkField::Merged(..) => {
				bail!("Attempted to parse merged array in blk-text function (array type is not available in the BLK format)")
			},
		}
	}
}

#[cfg(test)]
mod test {
	use std::sync::Arc;

	use crate::blk::{blk_structure::BlkField, blk_type::BlkType};
	use crate::blk::util::blk_str;

	#[test]
	fn test_expected() {
		// For testing purposes i should probably make a better way for this
		let mut root = BlkField::new_root();
		root.insert_field(BlkField::Value(
			blk_str("vec4f"),
			BlkType::Float4([1.25, 2.5, 5.0, 10.0]),
		))
		.unwrap();
		root.insert_field(BlkField::Value(
			blk_str("int"),
			BlkType::Int(42),
		))
		.unwrap();
		root.insert_field(BlkField::Value(
			blk_str("long"),
			BlkType::Long(42),
		))
		.unwrap();

		let mut alpha = BlkField::new_struct(blk_str("alpha"));
		alpha
			.insert_field(BlkField::Value(
				blk_str("str"),
				BlkType::Str(blk_str("hello")),
			))
			.unwrap();
		alpha
			.insert_field(BlkField::Value(
				blk_str("bool"),
				BlkType::Bool(true),
			))
			.unwrap();
		alpha
			.insert_field(BlkField::Value(
				blk_str("color"),
				BlkType::Color {
					r: 1,
					g: 2,
					b: 3,
					a: 4,
				},
			))
			.unwrap();

		let mut gamma = BlkField::new_struct(blk_str("gamma"));
		gamma
			.insert_field(BlkField::Value(
				blk_str("vec2i"),
				BlkType::Int2([3, 4]),
			))
			.unwrap();
		gamma
			.insert_field(BlkField::Value(
				blk_str("vec2f"),
				BlkType::Float2([1.25, 2.5]),
			))
			.unwrap();
		gamma
			.insert_field(BlkField::Value(
				blk_str("transform"),
				BlkType::Float12(Box::new([
					1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.25, 2.5, 5.0,
				])),
			))
			.unwrap();
		alpha.insert_field(gamma).unwrap();
		root.insert_field(alpha).unwrap();

		let mut beta = BlkField::new_struct(blk_str("beta"));
		beta.insert_field(BlkField::Value(
			blk_str("float"),
			BlkType::Float(1.25),
		))
		.unwrap();
		beta.insert_field(BlkField::Value(
			blk_str("vec2i"),
			BlkType::Int2([1, 2]),
		))
		.unwrap();
		beta.insert_field(BlkField::Value(
			blk_str("vec3f"),
			BlkType::Float3([1.25, 2.5, 5.0]),
		))
		.unwrap();
		root.insert_field(beta).unwrap();

		println!("{}", root.inner_as_blk_text(&mut 0, true).unwrap());
	}
}
