use color_eyre::eyre::bail;
use color_eyre::Report;
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
			BlkField::Value(name, value) => {
				Ok(format!("\"{name}\":{value}"))
			},
			BlkField::Struct(name, fields) => {
				let indent = "\t".repeat(*indent_level);
				*indent_level += 1;
				let children = fields
					.iter()
					.map(|x|Ok( format!("{indent}{}", x.inner_as_blk_text(indent_level, false)?)))
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
			}
		}
	}
}

#[cfg(test)]
mod test {
	use std::sync::Arc;

	use crate::blk::{blk_structure::BlkField, blk_type::BlkType};

	#[test]
	fn test_expected() {
		// For testing purposes i should probably make a better way for this
		let mut root = BlkField::new_root();
		root.insert_field(BlkField::Value(
			Arc::new("vec4f".to_owned()),
			BlkType::Float4([1.25, 2.5, 5.0, 10.0]),
		))
		.unwrap();
		root.insert_field(BlkField::Value(Arc::new("int".to_owned()), BlkType::Int(42)))
			.unwrap();
		root.insert_field(BlkField::Value(
			Arc::new("long".to_owned()),
			BlkType::Long(42),
		))
		.unwrap();

		let mut alpha = BlkField::new_struct(Arc::new("alpha".to_owned()));
		alpha
			.insert_field(BlkField::Value(
				Arc::new("str".to_owned()),
				BlkType::Str(Arc::new("hello".to_owned())),
			))
			.unwrap();
		alpha
			.insert_field(BlkField::Value(
				Arc::new("bool".to_owned()),
				BlkType::Bool(true),
			))
			.unwrap();
		alpha
			.insert_field(BlkField::Value(
				Arc::new("color".to_owned()),
				BlkType::Color {
					r: 1,
					g: 2,
					b: 3,
					a: 4,
				},
			))
			.unwrap();

		let mut gamma = BlkField::new_struct(Arc::new("gamma".to_owned()));
		gamma
			.insert_field(BlkField::Value(
				Arc::new("vec2i".to_owned()),
				BlkType::Int2([3, 4]),
			))
			.unwrap();
		gamma
			.insert_field(BlkField::Value(
				Arc::new("vec2f".to_owned()),
				BlkType::Float2([1.25, 2.5]),
			))
			.unwrap();
		gamma
			.insert_field(BlkField::Value(
				Arc::new("transform".to_owned()),
				BlkType::Float12(Box::new([
					1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.25, 2.5, 5.0,
				])),
			))
			.unwrap();
		alpha.insert_field(gamma).unwrap();
		root.insert_field(alpha).unwrap();

		let mut beta = BlkField::new_struct(Arc::new("beta".to_owned()));
		beta.insert_field(BlkField::Value(
			Arc::new("float".to_owned()),
			BlkType::Float(1.25),
		))
		.unwrap();
		beta.insert_field(BlkField::Value(
			Arc::new("vec2i".to_owned()),
			BlkType::Int2([1, 2]),
		))
		.unwrap();
		beta.insert_field(BlkField::Value(
			Arc::new("vec3f".to_owned()),
			BlkType::Float3([1.25, 2.5, 5.0]),
		))
		.unwrap();
		root.insert_field(beta).unwrap();

		println!("{}", root.inner_as_blk_text(&mut 0, true).unwrap());
	}
}
