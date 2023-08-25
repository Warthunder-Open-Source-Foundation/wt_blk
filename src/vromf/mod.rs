mod de_obfuscation;
mod enums;
mod util;

/// This module unpacks the "outer" shell of the vromf image
/// It is generally expected to directly call into the public interfaces from this module, ignoring the inner lower-level functions
mod binary_container;

/// Unpacks the contents after binary unpacking
mod inner_container;
#[cfg(test)]
mod test;
mod unpacker;

pub use {
	unpacker::VromfUnpacker,
	unpacker::BlkOutputFormat,
	unpacker::File,
};
