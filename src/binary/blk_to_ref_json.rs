use crate::binary::blk_structure::BlkField;
use crate::binary::blk_type::{BlkCow, BlkType};
use crate::binary::output_formatting_conf::FormattingConfiguration;

impl BlkField<'_> {
	// Public facing formatting fn
	pub fn as_ref_json(&self, fmt: FormattingConfiguration) -> String {
		let mut initial_indentation = if fmt.global_curly_bracket {
			1
		} else {
			0
		};
		self._as_ref_json(&mut initial_indentation, true, fmt)
	}

	// TODO: Make this generic with a configuration file
	// Internal fn that actually formats
	fn _as_ref_json(&self, indent_level: &mut usize, is_root: bool, fmt: FormattingConfiguration) -> String {
		match self {
			BlkField::Value(name, value) => {
				format!("\"{name}\": {},", value.as_ref_json(fmt))
			}
			BlkField::Struct(name, fields) => {
				let mut indent = fmt.indent(*indent_level);
				*indent_level += 1;
				let children = fields.iter().map(|x| format!("{indent}{}", x._as_ref_json(indent_level, false, fmt))).collect::<Vec<_>>().join("\n");
				*indent_level -= 1;

				let indent_closing = fmt.indent(indent_level.saturating_sub(1));
				if is_root {
					if fmt.global_curly_bracket {
						format!("{{\n{}}}", children)
					} else {
						format!("{children}")
					}
				} else {
					// Empty blocks will not be opened or indented
					let block_delimiter = if fields.len() == 0 {
						indent = "".to_owned();
						""
					} else {
						"\n"
					};
					let name_delimiter = if fmt.object_colon {
						":"
					} else {
						""
					};
					format!("\"{name}\"{name_delimiter} {{{block_delimiter}{children}{block_delimiter}{indent_closing}}}")

				}
			}
		}
	}
}
