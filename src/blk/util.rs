use std::ffi::OsStr;

use crate::{
	blk::file::FileType,
	vromf::File,
};
#[inline(always)]
pub(crate) fn bytes_to_offset(input: &[u8]) -> Option<usize> {
	if input.len() != 4 {
		return None;
	}

	Some(u32::from_le_bytes([input[0], input[1], input[2], input[3]]) as usize)
}

#[inline(always)]
pub(crate) fn bytes_to_float(input: &[u8]) -> Option<f32> {
	if input.len() != 4 {
		return None;
	}

	Some(f32::from_le_bytes([input[0], input[1], input[2], input[3]]))
}

#[inline(always)]
pub(crate) fn bytes_to_int(input: &[u8]) -> Option<i32> {
	if input.len() != 4 {
		return None;
	}

	Some(i32::from_le_bytes([input[0], input[1], input[2], input[3]]))
}

#[inline(always)]
pub(crate) fn bytes_to_uint(input: &[u8]) -> Option<u32> {
	if input.len() != 4 {
		return None;
	}

	Some(u32::from_le_bytes([input[0], input[1], input[2], input[3]]))
}

#[inline(always)]
pub(crate) fn bytes_to_long(input: &[u8]) -> Option<i64> {
	if input.len() != 8 {
		return None;
	}

	Some(i64::from_le_bytes([
		input[0], input[1], input[2], input[3], input[4], input[5], input[6], input[7],
	]))
}

/// Simple check to differentiate plaintext BLK from binary one
pub fn maybe_blk(file: &File) -> bool {
	file.path().extension() == Some(OsStr::new("blk"))
		&& file.buf().len() > 0
		&& FileType::from_byte(file.buf()[0]).is_ok()
}
