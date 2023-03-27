use crate::binary::error::ParseError;

// Yields length in buffer and value
#[inline]
pub fn uleb128(bytes: &[u8]) -> Result<(usize, usize), ParseError> {
	let mut result = 0u128;
	let mask = 1 << 7;

	for i in 0..bytes.len() {
		let bits = (bytes[i] & (mask - 1)) as u128;
		result |=  bits << (7 * i);

		if mask & bytes[i] == 0 {
			return Ok((i + 1, result as usize));
		};
	}
	if bytes.len() == 0 {
		Err(ParseError::ZeroSizedUleb)
	} else {
		Err(ParseError::UnexpectedEndOfBufferUleb)
	}
}