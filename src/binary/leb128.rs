// Yields length in buffer and value
pub fn uleb128(bytes: &[u8]) -> Option<(usize, usize)> {
	let mut result = 0u128;
	let mask = 1 << 7;

	for i in 0..bytes.len() {
		let bits = (bytes[i] & (mask - 1)) as u128;
		result |=  bits << (7 * i);

		if mask & bytes[i] == 0 {
			return Some((i + 1, result as usize));
		};
	}
	None
}