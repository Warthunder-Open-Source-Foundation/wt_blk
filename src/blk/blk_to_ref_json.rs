use std::collections::HashMap;
use std::fmt::Write;

use crate::{
	blk::{blk_structure::BlkField, output_formatting_conf::FormattingConfiguration},
	vromf::VromfError,
};
use crate::blk::util::indent;

/// Reference JSON is an output format dedicated to mirroring the behaviour of existing formatters

impl BlkField {
	// Public facing formatting fn
	pub fn as_ref_json(&self, fmt: FormattingConfiguration) -> Result<String, VromfError> {
		let mut writer = String::with_capacity(self.estimate_size() * 3); // Prealloc a rough guesstimate of this structs size
		let mut initial_indentation = if fmt.global_curly_bracket { 1 } else { 0 };
		self._as_ref_json(&mut writer, &mut initial_indentation, true, fmt, true)?;
		Ok(writer)
	}

	// Indent level decides how deeply nested the current fields are
	// The root flag decides special conditions for global output
	// The formatting configuration sets some preferences for output
	// Is last elem prevents trailing commas when the current element is the last of an object
	fn _as_ref_json(
		&self,
		f: &mut String,
		indent_level: &mut usize,
		is_root: bool,
		fmt: FormattingConfiguration,
		is_last_elem: bool,
	) -> Result<(), VromfError> {
		let trail_comma = if is_last_elem { "" } else { "," };
		match self {
			BlkField::Value(name, value) => {
				write!(f, "\"{name}\": ")?;
				value.as_ref_json(f, fmt, *indent_level)?;
				write!(f, "{trail_comma}")?;
			},
			BlkField::Struct(name, fields) => {
				let indent_count = *indent_level;
				let indent_closing_count = indent_level.saturating_sub(1);
				if is_root {
					// Prepend global brackets
					if fmt.global_curly_bracket {
						write!(f, "{{\n")?;
					}

					render_fields(fields, f, indent_level,  indent_count,fmt)?;

					// Append global brackets
					if fmt.global_curly_bracket {
						write!(f, "\n}}")?;
					}
				} else {
					// Empty blocks will not be opened or indented
					let block_delimiter = if fields.len() == 0 { "" } else { "\n" };

					// An object might be formatted as such: `object: {}` or as `object {}`
					let name_delimiter = if fmt.object_colon { ":" } else { "" };

					//                                     v Opening bracket
					write!(f, "\"{name}\"{name_delimiter} {{{block_delimiter}")?;
					render_fields(fields, f, indent_level, indent_count, fmt)?;
					write!(f, "{block_delimiter}")?;
					indent(f, indent_closing_count, fmt.indent_char)?;
					//         v Closing bracket
					write!(f, "}}{trail_comma}")?;
				}
			},
		}
		Ok(())
	}
}

fn render_fields(fields: &Vec<BlkField>, f: &mut String, indent_level: &mut usize, base_indentation: usize, fmt: FormattingConfiguration) -> Result<(), VromfError> {
	*indent_level += 1;
	for (i, field) in fields.iter().enumerate() {
		indent(f, base_indentation, fmt.indent_char)?; // Indent for next children
		field._as_ref_json(f, indent_level, false, fmt, i == fields.len() - 1)?;
		if fields.len() - 1 != i {
			write!(f, "\n")?; // Trailing newline, unless last
		}
	}
	*indent_level -= 1;
	Ok(())
}

#[cfg(test)]
mod test {
	use std::{
		fs,
		path::{Path, PathBuf},
		str::FromStr,
		time::Instant,
	};

	use crate::{
		blk::{output_formatting_conf::FormattingConfiguration, BlkOutputFormat},
		vromf::unpacker::VromfUnpacker,
	};

	// #[test]
	fn test_newline_parity() {
		let referece = fs::read_to_string("./samples/login_bkg_1_63_nolayers_jp.blk").unwrap();
		let aces = fs::read("./samples/aces.vromfs.bin").unwrap();
		let parsed = VromfUnpacker::from_file((
			PathBuf::from_str("./samples/aces.vromfs.bin").unwrap(),
			aces,
		))
		.unwrap()
		.unpack_all(Some(BlkOutputFormat::Json(
			FormattingConfiguration::GSZABI_REPO,
		)))
		.unwrap();
		let needed = parsed
			.iter()
			.filter(|e| e.0.ends_with("login_bkg_1_63_nolayers_jp.blk"))
			.next()
			.unwrap()
			.to_owned();
		assert_eq!(String::from_utf8(needed.1).unwrap(), referece);
	}

	// 550ms
	#[test]
	fn perf_all() {
		let unpacker = VromfUnpacker::from_file((
			PathBuf::from_str("aces.vromfs.bin").unwrap(),
			include_bytes!("../../samples/aces.vromfs.bin").to_vec(),
		))
		.unwrap();
		let start = Instant::now();
		unpacker
			.unpack_all(Some(BlkOutputFormat::Json(
				FormattingConfiguration::GSZABI_REPO,
			)))
			.unwrap();
		println!("{:?}", start.elapsed());
	}

	#[test]
	fn parity_once() {
		let unpacker = VromfUnpacker::from_file((
			PathBuf::from_str("aces.vromfs.bin").unwrap(),
			include_bytes!("../../samples/aces.vromfs.bin").to_vec(),
		))
		.unwrap();
		let start = Instant::now();
		let unpacked = unpacker
			.unpack_one(
				Path::new("gamedata/weapons/rocketguns/fr_r_550_magic_2.blk"),
				Some(BlkOutputFormat::Json(FormattingConfiguration::GSZABI_REPO)),
			)
			.unwrap();
		println!("{:?}", start.elapsed());

		let reference = fs::read("./samples/magic_2_json_baseline.json").unwrap();
		assert_eq!(
			String::from_utf8(unpacked).unwrap(),
			String::from_utf8(reference).unwrap()
		);
	}
}
