use color_eyre::{Report, eyre::bail};

use crate::blk::{blk_string::BlkString, blk_structure::BlkField, blk_type::BlkFormatting};

impl BlkField {
	// Public facing formatting fn
	pub fn as_blk_text(&self, format: BlkFormatting) -> Result<String, Report> {
		self.inner_as_blk_text(&mut 0, true, format)
	}

	// TODO: Make this generic with a configuration file
	// Internal fn that actually formats
	fn inner_as_blk_text(
		&self,
		indent_level: &mut usize,
		is_root: bool,
		format: BlkFormatting,
	) -> Result<String, Report> {
		match self {
			BlkField::Value(name, value) => Ok(format!(
				"{name}:{value}",
				name = quote_name(name),
				value = value.to_string_with(format)
			)),
			BlkField::Struct(name, fields) => {
				let indent = "\t".repeat(*indent_level);
				*indent_level += 1;
				let children = fields
					.iter()
					.map(|x| {
						Ok(format!(
							"{indent}{}",
							x.inner_as_blk_text(indent_level, false, format)?
						))
					})
					.collect::<Result<Vec<_>, Report>>()?
					.join("\n");
				*indent_level -= 1;

				let indent_closing = "\t".repeat(indent_level.saturating_sub(1));
				Ok(if is_root {
					format!("{children}")
				} else {
					format!("{} {{\n{children}\n{indent_closing}}}", quote_name(name))
				})
			},
			BlkField::Merged(..) => {
				bail!(
					"Attempted to parse merged array in blk-text function (array type is not available in the BLK format)"
				)
			},
		}
	}
}

/// Names made only of Dagor identifier characters are written bare, anything else is quoted,
/// like `dblk::is_ident_char` in Dagor's `blk_serialize.cpp`
fn quote_name(name: &BlkString) -> String {
	let is_ident = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '~');
	if !name.is_empty() && name.chars().all(is_ident) {
		name.to_string()
	} else {
		quote_string(name)
	}
}

/// Quotes a string the way Dagor's `writeStringValue` does, so its text parser reads it back
/// unchanged: `~` is the escape character (`~~`, `~"`, `~'`, `~t`, `~n`, `~r`), backslashes are
/// literal, a string holding `"` but no `'` is put in single quotes, and strings with line breaks
/// use triple quotes
pub(crate) fn quote_string(s: &str) -> String {
	let has_ln = s.contains(['\n', '\r']);
	let has_quot = s.contains('"');
	let has_tick = s.contains('\'');
	let (open, close) = if has_ln && (!has_quot || has_tick) {
		("\"\"\"\n", "\n\"\"\"")
	} else if has_ln {
		("'''\n", "\n'''")
	} else if has_quot && !has_tick {
		("'", "'")
	} else {
		("\"", "\"")
	};
	let single = open.len() == 1;
	let q = open.chars().next().unwrap();
	let mut out = String::with_capacity(s.len() + 2);
	out.push_str(open);
	let mut chars = s.chars().peekable();
	while let Some(c) = chars.next() {
		match c {
			'~' => out.push_str("~~"),
			c if c == q && (single || chars.peek() == Some(&c)) => {
				out.push('~');
				out.push(c);
			},
			'\r' if single => out.push_str("~r"),
			'\n' if single => out.push_str("~n"),
			'\t' => out.push_str("~t"),
			c => out.push(c),
		}
	}
	out.push_str(close);
	out
}

#[cfg(test)]
mod test {
	use crate::blk::{
		blk_string::blk_str,
		blk_structure::BlkField,
		blk_type::{BlkFormatting, BlkType},
		make_strict_test,
	};

	#[test]
	fn test_expected() {
		// For testing purposes i should probably make a better way for this
		let root = make_strict_test();
		println!(
			"{}",
			root.inner_as_blk_text(&mut 0, true, BlkFormatting::standard())
				.unwrap()
		);
	}

	#[test]
	fn test_escaping() {
		let root = BlkField::Value(
			blk_str("totally not escaped"),
			BlkType::Str(blk_str("this is totally not escaped \" ")),
		);
		assert_eq!(
			root.inner_as_blk_text(&mut 0, true, BlkFormatting::standard())
				.unwrap(),
			r#""totally not escaped":t = 'this is totally not escaped " '"#
		);
	}

	#[test]
	fn strings_and_names_use_dagor_escapes() {
		let value = |name: &str, s: &str| BlkField::Value(blk_str(name), BlkType::Str(blk_str(s)));
		let root = BlkField::Struct(
			blk_str("root"),
			vec![
				value("path", r"..\..\prog\scripts"),
				value("anim", "~hatch_hide"),
				value("quoted", r#"say "hi""#),
				value("both", r#"it's "x""#),
				value("tab", "a\tb"),
				value("lines", "one\ntwo"),
				value("@override", ""),
				BlkField::Value(blk_str("rendinst::tm"), BlkType::Bool(true)),
				BlkField::Struct(blk_str("a b"), vec![]),
			],
		);
		let expected = [
			r#"path:t="..\..\prog\scripts""#,
			r#"anim:t="~~hatch_hide""#,
			r#"quoted:t='say "hi"'"#,
			r#"both:t="it's ~"x~"""#,
			r#"tab:t="a~tb""#,
			"lines:t=\"\"\"\none\ntwo\n\"\"\"",
			r#""@override":t="""#,
			r#""rendinst::tm":b=yes"#,
			"\"a b\" {\n\n}",
		];
		assert_eq!(
			root.as_blk_text(BlkFormatting::compact()).unwrap(),
			expected.join("\n")
		);
	}
}
