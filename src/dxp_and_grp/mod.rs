use std::fs::File;

use crate::dxp_and_grp::{
	dxp::parse_dxp,
	error::{DxpGrpError, DxpGrpError::Utf8Error},
	grp::parse_grp,
};

mod dxp;
mod error;
mod grp;

pub fn parse_either(file: &[u8]) -> Result<Vec<String>, DxpGrpError> {
	if file.len() < 4 {
		return Err(DxpGrpError::FileTooShort { len: file.len() });
	}

	match <&[u8; 4]>::try_from(&file[0..4]).expect("Infallible") {
		b"GRP2" => parse_grp(file),
		b"DxP2" => parse_dxp(file),
		_ => Err(DxpGrpError::InvalidHeader {
			found: String::from_utf8(file[0..4].to_owned())
				.map_err(|e| Utf8Error(e.utf8_error()))?,
		}),
	}
}

/// This function yields the names from a DXP file, using a relative OS buffer
/// # Safety
/// Do not modify or change the file during this call, as it will result in UB
/// It is recommended to open the file in Read-Write mode, to avoid  invalid external accesses
pub fn parse_buffered(file: &File) -> Result<Vec<String>, DxpGrpError> {
	let file = unsafe { memmap2::Mmap::map(file)? };

	parse_either(&file)
}

#[cfg(test)]
mod test {
	use std::{fs, fs::File};

	use crate::dxp_and_grp::{dxp::parse_dxp, parse_buffered};

	#[test]
	fn fat_hq_tex() {
		let f = fs::read("./samples/dxp/hq_tex_water_garbage_piles.dxp.bin").unwrap();
		let _out = parse_dxp(&f).unwrap();
	}

	#[test]
	fn fat_hq_tex_buffered() {
		let _out = parse_buffered(
			&File::open("./samples/dxp/hq_tex_water_garbage_piles.dxp.bin").unwrap(),
		)
		.unwrap();
	}
}
