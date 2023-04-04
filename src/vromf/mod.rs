use crate::binary::util::bytes_to_int;

pub const VRFS: u32 = 0x73465256;
// 'VRFs' in binary, stands for simple header
pub const VRFX: u32 = 0x78465256; // ' VRFx' in binary, stands for extended header

pub mod platform_type {
	pub const PC: u32 = 0x43500000;
	// b'\x00\x00PC'
	pub const IOS: u32 = 0x534f6900;
	// b'\x00iOS'
	pub const ANDROID: u32 = 0x646e6100; // b'\x00and'
}

// While stored as bytes, the true encoding is just 6 bits
pub const ZSTD_OBFS_NOCHECK: u8 = 0x10;
// ZSTD compressed and obfuscated. No digest
pub const PLAIN: u8 = 0x20;
// Image in plain form. With digest
pub const ZSTD_OBFS: u8 = 0x30; // Same as ZSTD_OBFS_NOCHECK except with digest


pub fn decode_bin_vromf(file: &[u8]) {
	let mut ptr = 0_usize;

	// Returns slice offset from file, incrementing the ptr by offset
	let idx_file_offset = |ptr: &mut usize, offset: usize| {
		let res = file.get(*ptr..(*ptr + offset)).unwrap();
		*ptr += offset;
		res
	};

	let header_type = bytes_to_int(idx_file_offset(&mut ptr, 4)).unwrap();
	let is_extended_header = match header_type {
		VRFS => false,
		VRFX => true,
		_ => {
			panic!("Ruh oh")
		}
	};
	println!("{} {:x}", is_extended_header, header_type);

	let platform = bytes_to_int(idx_file_offset(&mut ptr, 4)).unwrap();

	// Just validating here
	match platform {
		platform_type::PC | platform_type::IOS | platform_type::ANDROID => {}
		_ => {
			panic!("Ruh oh")
		}
	}
	println!("{:x}", platform);

	let size = bytes_to_int(idx_file_offset(&mut ptr, 4)).unwrap();
	println!("{}", size);

	let header_packed = bytes_to_int(idx_file_offset(&mut ptr, 4)).unwrap();
	const TYPE_MASK: u32 = 0b11111100000000;
	const SIZE_MASK: u32 = !TYPE_MASK;
	println!("{:?}", header_packed.to_le_bytes().map(|x|format!("{x:x}")));
	let pack_type = ((header_packed & TYPE_MASK) >> 26) as u8;
	println!("{}", pack_type);

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