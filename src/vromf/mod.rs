use crate::vromf::binary_container::decode_bin_vromf;
use crate::vromf::error::VromfError;
use crate::vromf::inner_container::decode_inner_vromf;

mod enums;
mod util;
mod de_obfuscation;

/// It is generally expected to directly call into the public interfaces from this module, ignoring the inner lower-level functions

// This module unpacks the "outer" shell of the vromf image
mod binary_container;

// This module unpacks the inner parts of the binary image
mod inner_container;
pub mod error;

pub fn decode_vromf(file: &[u8]) -> Result<Vec<(String, Vec<u8>)>, VromfError> {
	let decoded = decode_bin_vromf(&file)?;
	let inner = decode_inner_vromf(&decoded);
	Ok(inner)
}