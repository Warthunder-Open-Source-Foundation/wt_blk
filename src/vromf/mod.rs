use crate::vromf::{
	binary_container::decode_bin_vromf,
	error::VromfError,
	inner_container::decode_inner_vromf,
};
use crate::vromf::binary_container::FileMode;

mod de_obfuscation;
pub mod enums;
mod util;

/// It is generally expected to directly call into the public interfaces from this module, ignoring the inner lower-level functions
// This module unpacks the "outer" shell of the vromf image
mod binary_container;

// This module unpacks the inner parts of the binary image
pub mod error;
mod inner_container;
#[cfg(test)]
mod test;

pub fn decode_vromf(file: &[u8], file_mode: FileMode) -> Result<Vec<(String, Vec<u8>)>, VromfError> {
	let decoded = decode_bin_vromf(&file, file_mode)?;
	let inner = decode_inner_vromf(&decoded)?;
	Ok(inner)
}
