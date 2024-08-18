/// De-obfuscates vromv stream and/or beginning
pub mod de_obfuscation;
mod enums;
mod util;

/// This module unpacks the "outer" shell of the vromf image
/// It is generally expected to directly call into the public interfaces from this module, ignoring the inner lower-level functions
mod binary_container;

pub(crate) mod file;
mod header;
/// Unpacks the contents after binary unpacking
mod inner_container;
#[cfg(test)]
mod test;
mod unpacker;

pub use file::File;
pub use unpacker::{BlkOutputFormat, VromfUnpacker};
