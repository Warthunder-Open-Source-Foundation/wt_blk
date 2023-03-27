use crate::binary::error::ParseError;

// Yields length in buffer and value
#[inline]
pub fn uleb128(bytes: &[u8]) -> Result<(usize, usize), ParseError> {
	let mut result = 0_usize;
	let mask = 1 << 7;

	for (i, current) in bytes.iter().enumerate() {
		let bits = (current & (mask - 1)) as usize;
		result |=  bits << (7 * i);

		if mask & current == 0 {
			return Ok((i + 1, result));
		};
	}
	if bytes.len() == 0 {
		Err(ParseError::ZeroSizedUleb)
	} else {
		Err(ParseError::UnexpectedEndOfBufferUleb)
	}
}