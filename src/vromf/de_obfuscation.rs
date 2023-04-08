use hex::FromHex;
use lazy_static::lazy_static;

// This magic sequence runs XOR over input to deobfuscate it
lazy_static!{
	static ref HEAD: Vec<u8> = {
		hex::decode(b"55aa55aa0ff00ff055aa55aa48124812").unwrap()
	};
	static ref TAIL: Vec<u8> = {
		hex::decode(b"4812481255aa55aa0ff00ff055aa55aa").unwrap()
	};
}

pub fn deob(input: &mut [u8]) {
	match input.len() {
		0..=15 => return,
		16..=31 => {
			xor_at_with(input, 0, &HEAD);
		}
		32.. => {
			xor_at_with(input, 0, &HEAD);
			let at = (input.len() & 0x03FF_FFFC) - 16;
			xor_at_with(input, at, &TAIL);
		}
		_ => {unreachable!()}
	}
}


fn xor_at_with(input: &mut [u8], at: usize, with: &Vec<u8>) {
	for (i, byte) in input[at..(at + 16)].iter_mut().enumerate() {
		*byte = *byte ^ with[i];
	}
}


#[cfg(test)]
mod test {
	use crate::vromf::de_obfuscation::{deob};

	#[test]
	pub fn test_24() {
		let mut start = vec![0xFF_u8; 24];
		deob(&mut start);

		let expected: &[u8] = &[
			0xAA, 0x55, 0xAA, 0x55, 0xF0, 0x0F, 0xF0, 0x0F,
			0xAA, 0x55, 0xAA, 0x55, 0xB7, 0xED, 0xB7, 0xED,
			0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
		];

		assert_eq!(&start, expected)
	}

	#[test]
	pub fn test_38() {
		let mut start = vec![0xFF_u8; 38];
		deob(&mut start);

		let expected: &[u8] = &[
			0xAA, 0x55, 0xAA, 0x55, 0xF0, 0x0F, 0xF0, 0x0F, 0xAA, 0x55, 0xAA, 0x55, 0xB7, 0xED, 0xB7, 0xED,
			0xFF, 0xFF, 0xFF, 0xFF, 0xB7, 0xED, 0xB7, 0xED, 0xAA, 0x55, 0xAA, 0x55, 0xF0, 0x0F, 0xF0, 0x0F,
			0xAA, 0x55, 0xAA, 0x55, 0xFF, 0xFF,
		];

		assert_eq!(&start, expected)
	}
}