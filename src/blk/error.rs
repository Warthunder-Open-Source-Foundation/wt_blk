#[derive(thiserror::Error, Debug)]
pub enum BlkTypeError {
	#[error("Attempted to parse {expected} from buffer len {found}")]
	NumberSizeMissmatch {
		found: usize,
		expected: &'static str,
	},
	#[error("BLK field should be 4 bytes, found {found}")]
	TypeFieldSizeMissmatch {
		found: usize,
	},
	#[error("Unknown BLK type code")]
	UnknownTypeId(u8),
}
