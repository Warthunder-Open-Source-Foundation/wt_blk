use std::{ffi::CStr, mem::size_of};

use crate::{
	blk::util::bytes_to_offset,
	dxp_and_grp::{
		dxp::DxpGrpError::{FileTooShort, InvalidHeader},
		error::{DxpGrpError, DxpGrpError::IndexingFileOutOfBounds},
	},
};

/// This function yields the names from a DXP file

// IF YOU WISH TO SEE THE CONTENTS OF DXP FILES, YOU SHOULD OPEN AN ISSUE AND I WILL ADD FUNCTIONALITY FOR IT
// The data section is pretty much similar to the inner-vromf container
// It has a section of offsets + sizes, followed by the data region indexed by said offsets and sizes

pub fn parse_dxp(file: &[u8]) -> Result<Vec<String>, DxpGrpError> {
	// Return empty names for empty file
	if file.len() == 0 {
		return Ok(vec![]);
	}
	if file.len() < 0x48 {
		return Err(FileTooShort { len: file.len() });
	}

	let dxp_header = String::from_utf8(file[0..4].to_owned()).map_err(|e| e.utf8_error())?;
	if dxp_header != "DxP2" {
		return Err(InvalidHeader { found: dxp_header });
	}

	// Fixed offset at 0x8
	let file_count = bytes_to_offset(&file.get(0x8..(0x8 + size_of::<u32>())).ok_or(
		IndexingFileOutOfBounds {
			current_ptr: 0x8,
			file_size:   file.len(),
		},
	)?)
	.expect("Infallible");

	// Names begin at 0x48, usual CString sequence
	let mut ptr: usize = 0x48;
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
