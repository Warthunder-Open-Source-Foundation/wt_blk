use color_eyre::{eyre::bail, Report};

use crate::blk::{blk_string::BlkString, blk_structure::BlkField, blk_type::BlkType};

impl BlkField {
	// Public facing formatting fn
	pub fn as_blk_text(&self) -> Result<String, Report> {
		self.inner_as_blk_text(&mut 0, true)
	}

	// TODO: Make this generic with a configuration file
	// Internal fn that actually formats
	fn inner_as_blk_text(&self, indent_level: &mut usize, is_root: bool) -> Result<String, Report> {
		match self {
			BlkField::Value(name, value) => Ok(format!(
				"{name}:{value}",
				name = escape_key(name),
				value = escape_value(value)
			)),
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
					format!("{name} {{\n{children}\n{indent_closing}}}")
				})
			},
			BlkField::Merged(..) => {
				bail!("Attempted to parse merged array in blk-text function (array type is not available in the BLK format)")
			},
		}
	}
}

fn escape_key(field: &BlkString) -> String {
	if field.contains(' ') {
		format!("\"{field}\"")
	} else {
		field.to_string()
	}
}

// Slowpath for escaping, it's a rare case
fn escape_value(value: &BlkType) -> String {
	match value {
		BlkType::Str(s) => {
			if s.contains('\"') {
				format!("{ty} = '{s}'", ty = value.blk_type_name())
			} else {
				value.to_string()
			}
		},
		_ => value.to_string(),
	}
}

#[cfg(test)]
mod test {
	use crate::blk::{
		blk_string::blk_str,
		blk_structure::BlkField,
		blk_type::BlkType,
		make_strict_test,
	};

	#[test]
	fn test_expected() {
		// For testing purposes i should probably make a better way for this
		let root = make_strict_test();
		println!("{}", root.inner_as_blk_text(&mut 0, true).unwrap());
	}

	#[test]
	fn test_escaping() {
		let root = BlkField::Value(
			blk_str("totally not escaped"),
			BlkType::Str(blk_str("this is totally not escaped \" ")),
		);
		assert_eq!(
			root.inner_as_blk_text(&mut 0, true).unwrap(),
			r#""totally not escaped":t = 'this is totally not escaped " '"#
		);
	}
}
