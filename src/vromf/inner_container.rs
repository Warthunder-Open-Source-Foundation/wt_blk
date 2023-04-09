pub fn decode_inner_vromf(file: &[u8]) {
	// Returns slice offset from file, incrementing the ptr by offset
	let idx_file_offset = |ptr: &mut usize, offset: usize| {
		let res = file.get(*ptr..(*ptr + offset)).unwrap();
		*ptr += offset;
		res
	};
	let mut ptr = 0;

	let container_type = idx_file_offset(&mut ptr, 1)[0];
	let has_digest = match container_type {
		0x20 => false,
		0x30 => true,
		_ => {panic!("uh oh, unknown container type")}
	};
}


#[cfg(test)]
mod test {
	use std::fs;
	use std::time::{Duration, Instant};
	use crate::util::time;
	use crate::vromf::binary_container::decode_bin_vromf;

	#[test]
	fn test_uncompressed() {
		let f = fs::read("./samples/checked_simple_uncompressed_checked.vromfs.bin").unwrap();
		let decoded = decode_bin_vromf(&f);
	}

	#[test]
	fn test_compressed() {
		let f = fs::read("./samples/unchecked_extended_compressed_checked.vromfs.bin").unwrap();
		let decoded = decode_bin_vromf(&f);
	}
}