use std::mem::size_of;

// This magic sequence runs XOR over input to deobfuscate it
const ZSTD_XOR_PATTERN: [u32; 4] = [0xAA55AA55, 0xF00FF00F, 0xAA55AA55, 0x12481248];
const ZSTD_XOR_PATTERN_REV: [u32; 4] = [
	ZSTD_XOR_PATTERN[3],
	ZSTD_XOR_PATTERN[2],
	ZSTD_XOR_PATTERN[1],
	ZSTD_XOR_PATTERN[0],
];

/// Unsets obfuscation bytes as defined by <https://github.com/GaijinEntertainment/DagorEngine/blob/main/prog/dagorInclude/supp/dag_zstdObfuscate.h>
pub fn deobfuscate(input: &mut [u8]) {
	match input.len() {
		0..=15 => return,
		16..=31 => {
			xor_at_with(input, 0, ZSTD_XOR_PATTERN);
		}
		32.. => {
			xor_at_with(input, 0, ZSTD_XOR_PATTERN);
			let at = (input.len() & 0x03FF_FFFC) - 16;
			xor_at_with(input, at, ZSTD_XOR_PATTERN_REV);
		}
		_ => {
			unreachable!()
		}
	}
}


// XORS sequence of 16 bytes from given starting point with 4x 32-bit u32
fn xor_at_with(input: &mut [u8], at: usize, with: [u32; 4]) {
	for (i, byte) in input[at..(at + 16)].iter_mut().enumerate() {
		*byte = *byte ^ with[i / 4].to_le_bytes()[i % size_of::<u32>()];
	}
}

#[cfg(test)]
mod test {
	use crate::vromf::de_obfuscation::deobfuscate;

	#[test]
	pub fn test_24() {
		let mut start = vec![0xFF_u8; 24];
		deobfuscate(&mut start);

		let expected: &[u8] = &[
			0xAA, 0x55, 0xAA, 0x55, 0xF0, 0x0F, 0xF0, 0x0F, 0xAA, 0x55, 0xAA, 0x55, 0xB7, 0xED,
			0xB7, 0xED, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
		];

		assert_eq!(&start, expected)
	}

	#[test]
	pub fn test_38() {
		let mut start = vec![0xFF_u8; 38];
		deobfuscate(&mut start);

		let expected: &[u8] = &[
			0xAA, 0x55, 0xAA, 0x55, 0xF0, 0x0F, 0xF0, 0x0F, 0xAA, 0x55, 0xAA, 0x55, 0xB7, 0xED,
			0xB7, 0xED, 0xFF, 0xFF, 0xFF, 0xFF, 0xB7, 0xED, 0xB7, 0xED, 0xAA, 0x55, 0xAA, 0x55,
			0xF0, 0x0F, 0xF0, 0x0F, 0xAA, 0x55, 0xAA, 0x55, 0xFF, 0xFF,
		];

		assert_eq!(&start, expected)
	}
}
