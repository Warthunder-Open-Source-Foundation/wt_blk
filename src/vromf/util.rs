use std::{mem::size_of, path::PathBuf};
use color_eyre::eyre::{bail, Context, ContextCompat};
use color_eyre::Report;
use crate::vromf::enums::Packing;


pub fn pack_type_from_aligned(input: u32) -> Option<(Packing, u32)> {
	const SIZE_MASK: u32 = 0b0000001111111111111111111111111;

	// Yields the first 6 bytes
	let pack_type_raw_aligned = (input.to_be_bytes()[0]) >> 2;
	let pack_type = Packing::try_from(pack_type_raw_aligned).ok()?;

	// yields the last 26 bytes
	let pack_size = input & SIZE_MASK;
	Some((pack_type, pack_size))
}

pub fn bytes_to_int(input: &[u8]) -> Result<u32, Report> {
	if input.len() != 4 {
		bail!("Expected buffer of length {}, found {}", size_of::<u32>(), input.len());
	}

	Ok(u32::from_le_bytes([input[0], input[1], input[2], input[3]]))
}

pub fn bytes_to_long(input: &[u8]) -> Result<u64, Report> {
	if input.len() != size_of::<u64>() {
		bail!("Expected buffer of length {}, found {}", size_of::<u64>(), input.len());
	}

	Ok(u64::from_le_bytes([
		input[0], input[1], input[2], input[3], input[4], input[5], input[6], input[7],
	]))
}

pub fn bytes_to_usize(input: &[u8]) -> Result<usize, Report> {
	let long = bytes_to_long(input)?;
	usize::try_from(long).context("64 bit integer did not fit into usize")
}

pub(crate) fn path_stringify(buf: &PathBuf) -> Result<String, Report> {
	buf.to_str().context("Failed to cast path {buf:?} into string").map(|e|e.to_string())
}
