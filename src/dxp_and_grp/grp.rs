use std::{ffi::CStr, mem::size_of};

use crate::{
	blk::util::bytes_to_offset,
	dxp_and_grp::error::{
		DxpGrpError,
		DxpGrpError::{FileTooShort, IndexingFileOutOfBounds, InvalidHeader},
	},
};

pub fn parse_grp(file: &[u8]) -> Result<Vec<String>, DxpGrpError> {
	if file.len() < 0x40 {
		return Err(FileTooShort { len: file.len() });
	}

	let grp_header = String::from_utf8(file[0..4].to_owned()).map_err(|e| e.utf8_error())?;
	if grp_header != "GRP2" {
		return Err(InvalidHeader { found: grp_header });
	}

	// Fixed offset at 0x8
	let file_count = bytes_to_offset(&file.get(0x14..(0x14 + size_of::<u32>())).ok_or(
		IndexingFileOutOfBounds {
			current_ptr: 0x24,
			file_size:   file.len(),
		},
	)?)
	.expect("Infallible");

	// Names begin at 0x40, usual CString sequence
	let mut ptr: usize = 0x40;
	let mut names = Vec::with_capacity(file_count);
	for _ in 0..file_count {
		let str = CStr::from_bytes_until_nul(&file.get(ptr..).ok_or(IndexingFileOutOfBounds {
			current_ptr: ptr,
			file_size:   file.len(),
		})?)?
		.to_str()?
		.to_owned();
		//              +1 for null
		ptr += str.len() + 1;
		names.push(str);
	}
	Ok(names)
}
