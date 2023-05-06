use crate::blk::{blk_structure::BlkField, output_formatting_conf::FormattingConfiguration};

/// Reference JSON is an output format dedicated to mirroring the behaviour of existing formatters

impl BlkField {
	// Public facing formatting fn
	pub fn as_ref_json(&self, fmt: FormattingConfiguration) -> String {
		let mut initial_indentation = if fmt.global_curly_bracket { 1 } else { 0 };
		self._as_ref_json(&mut initial_indentation, true, fmt, true)
	}

	// Indent level decides how deeply nested the current fields are
	// The root flag decides special conditions for global output
	// The formatting configuration sets some preferences for output
	// Is last elem prevents trailing commas when the current element is the last of an object
	fn _as_ref_json(
		&self,
		indent_level: &mut usize,
		is_root: bool,
		fmt: FormattingConfiguration,
		is_last_elem: bool,
	) -> String {
		let trail_comma = if is_last_elem { "" } else { "," };
		match self {
			BlkField::Value(name, value) => {
				format!(
					"\"{name}\": {}{trail_comma}",
					value.as_ref_json(fmt, *indent_level)
				)
			},
			BlkField::Struct(name, fields) => {
				let mut indent = fmt.indent(*indent_level);
				*indent_level += 1;
				let children = fields
					.iter()
					.enumerate()
					.map(|(i, x)| {
						format!(
							"{indent}{}",
							x._as_ref_json(indent_level, false, fmt, i == fields.len() - 1)
						)
					})
					.collect::<Vec<_>>()
					.join("\n");
				*indent_level -= 1;

				let indent_closing = fmt.indent(indent_level.saturating_sub(1));
				if is_root {
					if fmt.global_curly_bracket {
						format!("{{\n{children}\n}}")
					} else {
						format!("{children}")
					}
				} else {
					// Empty blocks will not be opened or indented
					let block_delimiter = if fields.len() == 0 { "" } else { "\n" };

					// An object might be formatted as such: `object: {}` or as `object {}`
					let name_delimiter = if fmt.object_colon { ":" } else { "" };
					format!("\"{name}\"{name_delimiter} {{{block_delimiter}{children}{block_delimiter}{indent_closing}}}{trail_comma}")
				}
			},
		}
	}
}

#[cfg(test)]
mod test {
	use std::fs;
	use std::path::{Path, PathBuf};
	use std::str::FromStr;
	use crate::blk::BlkOutputFormat;
	use crate::blk::output_formatting_conf::FormattingConfiguration;
	use crate::vromf::unpacker::VromfUnpacker;

	// #[test]
	fn test_newline_parity() {
		let referece = fs::read_to_string("./samples/login_bkg_1_63_nolayers_jp.blk").unwrap();
		let aces = fs::read("./samples/aces.vromfs.bin").unwrap();
		let parsed = VromfUnpacker::from_file((PathBuf::from_str("./samples/aces.vromfs.bin").unwrap(),aces)).unwrap().unpack_all(Some(BlkOutputFormat::Json(FormattingConfiguration::GSZABI_REPO))).unwrap();
		let needed = parsed.iter().filter(|e|e.0.ends_with("login_bkg_1_63_nolayers_jp.blk")).next().unwrap().to_owned();
		assert_eq!(String::from_utf8(needed.1).unwrap(), referece);
	}
}
