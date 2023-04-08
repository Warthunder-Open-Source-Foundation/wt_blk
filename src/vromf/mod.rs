mod enums;
mod util;

use crate::binary::util::bytes_to_int;
use crate::vromf::enums::{HeaderType, PlatformType};
use crate::vromf::util::pack_type_from_aligned;


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
	let (pack_type, size) = pack_type_from_aligned(header_packed).unwrap();
	println!("{:?}", pack_type);
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