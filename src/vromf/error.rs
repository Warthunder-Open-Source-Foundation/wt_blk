#[derive(Debug, thiserror::Error)]
pub enum VromfError {
	#[error("Expected buffer of length {expected_size}, found {found_buff:?}")]
	InvalidIntegerBuffer {
		expected_size: usize,
		found_buff: Vec<u8>,
	},

	#[error("The integer {found} is not a valid header")]
	InvalidHeaderType {
		found: u32,
	},

	#[error("The integer {found} is not a valid platform-type")]
	InvalidPlatformType {
		found: u32
	},

	#[error("The byte {found:X} is not a valid vromf-packing-configuration")]
	InvalidPackingConfiguration {
		found: u8,
	},
}