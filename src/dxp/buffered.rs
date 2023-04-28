
use core::ffi::FromBytesUntilNulError;
use std::{ffi::CStr, mem::size_of, str::Utf8Error};
use std::fs::File;
use std::io::{BorrowedCursor, BufRead, BufReader, Read, Seek};

use crate::{
	blk::util::bytes_to_offset,
	dxp::DxpError::{FileTooShort, IndexingFileOutOfBounds, NotADxp},
};
use crate::dxp::DxpError;

/// This function yields the names from a DXP file, using a relative 8kib buffer

pub fn parse_dxp_buffered(file: &File) -> Result<Vec<String>, DxpError> {
	let buf = BufReader::new(file);
	let file = buf.see

	// Return empty names for empty file
	if file.len() == 0 {
		return Ok(vec![]);
	}
	if file.len() < 0x48 {
		return Err(FileTooShort { len: file.len() });
	}

	let dxp_header = String::from_utf8(file[0..4].to_owned()).map_err(|e| e.utf8_error())?;
	if dxp_header != "DxP2" {
		return Err(NotADxp { found: dxp_header });
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

#[cfg(test)]
mod test {
	use std::fs;

	use crate::dxp::parse_dxp;

	#[test]
	fn fat_hq_tex() {
		let f = fs::read("./samples/dxp/hq_tex_water_garbage_piles.dxp.bin").unwrap();
		let _out = parse_dxp(&f).unwrap();
	}
}