use crate::blk::{
	blk_structure::BlkField,
	output_formatting_conf::FormattingConfiguration,
};

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
						format!("{{\n{children}}}")
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

					// An object might be formatted as such: `object: {}` or as `object {}`
					let name_delimiter = if fmt.object_colon { ":" } else { "" };
					format!("\"{name}\"{name_delimiter} {{{block_delimiter}{children}{block_delimiter}{indent_closing}}}{trail_comma}")
				}
			},
		}
	}
}
