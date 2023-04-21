use core::ffi::FromBytesUntilNulError;
use std::{ffi::CStr, mem::size_of, ops::Index, str::Utf8Error, string::FromUtf8Error};

use crate::{
	blk::util::{bytes_to_int, bytes_to_offset},
	dxp::DxpError::IndexingFileOutOfBounds,
};

/// IF YOU WISH TO SEE THE CONTENTS OF DXP FILES, YOU SHOULD OPEN AN ISSUE AND I WILL ADD FUNCTIONALITY FOR IT
/// The data section is pretty much similar to the inner-vromf container
/// It has a section of offsets + sizes, followed by the data region indexed by said offsets and sizes

pub fn parse_dxp(file: &[u8]) -> Result<Vec<String>, DxpError> {
	// Return empty names for empty file
	let dxp_header = match String::from_utf8(file[0..4].to_owned()) {
		Ok(header) => header,
		Err(_) => {
			return Ok(vec![]);
		},
	};
	if dxp_header != "DxP2" {
		panic!("Guh, this is not a dxp idiot")
	}

	// Fixed offset at 0x8
	let file_count = bytes_to_offset(&file.get(0x8..(0x8 + size_of::<u32>())).ok_or(
		IndexingFileOutOfBounds {
			current_ptr: 0x8,
			file_size:   file.len(),
		},
	)?)
	.unwrap();

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

#[derive(Clone, Debug, thiserror::Error)]
pub enum DxpError {
	#[error("The files header indicates it is not a DXP")]
	NotADxp,

	#[error(transparent)]
	CStringError(#[from] FromBytesUntilNulError),

	#[error(transparent)]
	Utf8Error(#[from] Utf8Error),

	#[error("current ptr {current_ptr} is out of bounds for file of size: {file_size}")]
	IndexingFileOutOfBounds {
		current_ptr: usize,
		file_size:   usize,
	},
}

#[cfg(test)]
mod test {
	use std::{fs, time::Instant};

	use crate::dxp::parse_dxp;

	#[test]
	fn fat_hq_tex() {
		let f = fs::read("./samples/dxp/hq_tex_water_garbage_piles.dxp.bin").unwrap();
		let out = parse_dxp(&f).unwrap();
	}
}
