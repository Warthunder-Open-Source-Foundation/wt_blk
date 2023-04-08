mod enums;

use crate::binary::util::bytes_to_int;
use crate::vromf::enums::{HeaderType, PlatformType};


pub fn decode_bin_vromf(file: &[u8]) {
	let mut ptr = 0_usize;

	// Returns slice offset from file, incrementing the ptr by offset
	let idx_file_offset = |ptr: &mut usize, offset: usize| {
		let res = file.get(*ptr..(*ptr + offset)).unwrap();
		*ptr += offset;
		res
	};

	let header_type = bytes_to_int(idx_file_offset(&mut ptr, 4)).unwrap();
	let is_extended_header = HeaderType::try_from(header_type).unwrap().is_extended();

	let platform_raw = bytes_to_int(idx_file_offset(&mut ptr, 4)).unwrap();
	let _platform = PlatformType::try_from(platform_raw).unwrap();

	let size = bytes_to_int(idx_file_offset(&mut ptr, 4)).unwrap();

	let header_packed: u32 = bytes_to_int(idx_file_offset(&mut ptr, 4)).unwrap();
	const SIZE_MASK: u32 = 0b0000001111111111111111111111111;

	// Yields the first 6 bytes
	let pack_type = (header_packed.to_be_bytes()[0]) >> 2;

	// yields the last 26 bytes
	let pack_size = header_packed & SIZE_MASK;


	println!("{:?}", header_packed.to_le_bytes().map(|x| format!("0x{x:X}")));
	println!("{:?} {pack_type:x}", pack_type.to_le_bytes().map(|x| format!("0x{x:X}")));
}

#[cfg(test)]
mod test {
	use std::fs;

	use crate::vromf::decode_bin_vromf;

	#[test]
	fn decode_simple() {
		let f = fs::read("./samples/checked_simple_uncompressed_checked.vromfs.bin").unwrap();
		decode_bin_vromf(&f);
	}

	#[test]
	fn decode_compressed() {
		let f = fs::read("./samples/unchecked_extended_compressed_checked.vromfs.bin").unwrap();
		decode_bin_vromf(&f);
	}
}