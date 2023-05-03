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
pub mod unpacker;

pub use crate::vromf::error::VromfError;
