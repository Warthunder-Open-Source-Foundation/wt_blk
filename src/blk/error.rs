/// This error is used in place of color_eyre::Report where performance is critical
/// Avoid using this error outside of hot-loops
#[derive(thiserror::Error, Debug)]
pub enum BlkTypeError {
	// Type specific


	#[error("Attempted to parse {expected} from buffer len {found}")]
	NumberSizeMissmatch {
		found: usize,
		expected: &'static str,
	},
	#[error("BLK field should be 4 bytes, found {found}")]
	TypeFieldSizeMissmatch {
		found: usize,
	},
	#[error("Unknown BLK type code {0}")]
	UnknownTypeId(u8),


	// Uleb specific

	#[error("Empty ULEB buffer")]
	EmptyBuffer,
	#[error("Buffer ended while continue bit was still set")]
	ReturnedDuringContinueBit,
}
