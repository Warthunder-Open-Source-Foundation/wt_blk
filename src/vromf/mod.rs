mod enums;
mod util;
mod de_obfuscation;

use std::fs;
use std::mem::size_of;
use crate::binary::util::bytes_to_int;
use crate::vromf::de_obfuscation::deob;
use crate::vromf::enums::{HeaderType, PlatformType};
use crate::vromf::util::pack_type_from_aligned;


pub fn decode_bin_vromf(file: &[u8]) -> Vec<u8> {
	let mut ptr = 0_usize;

	// Returns slice offset from file, incrementing the ptr by offset
	let idx_file_offset = |ptr: &mut usize, offset: usize| {
		let res = file.get(*ptr..(*ptr + offset)).unwrap();
		*ptr += offset;
		res
	};

	let header_type = bytes_to_int(idx_file_offset(&mut ptr, 4)).unwrap();
	let header_type = HeaderType::try_from(header_type).unwrap();

	let platform_raw = bytes_to_int(idx_file_offset(&mut ptr, 4)).unwrap();
	let _platform = PlatformType::try_from(platform_raw).unwrap();

	let size = bytes_to_int(idx_file_offset(&mut ptr, 4)).unwrap();

	let header_packed: u32 = bytes_to_int(idx_file_offset(&mut ptr, 4)).unwrap();
	let (pack_type, extended_header_size) = pack_type_from_aligned(header_packed).unwrap();

	let inner_data = if header_type.is_extended() {
		let extended_header_section = idx_file_offset(&mut ptr, size_of::<u16>() + size_of::<u16>() + size_of::<u32>());

		// Unused header elements, for now
		let _header_size = u16::from_le_bytes([extended_header_section[0], extended_header_section[1]]);
		let _flags = u16::from_le_bytes([extended_header_section[2], extended_header_section[3]]);
		let _version = (extended_header_section[7],extended_header_section[6],extended_header_section[5],extended_header_section[4]);

		idx_file_offset(&mut ptr, extended_header_size as usize)

	} else {
		idx_file_offset(&mut ptr, size as usize)
	};

	// Directly return when data is not obfuscated
	if !pack_type.is_obfuscated() {
		return inner_data.to_vec()
	}

	let mut output = Vec::with_capacity(inner_data.len());
	deob(&mut output);

	output
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