use std::mem::size_of;

use crate::vromf::{
	de_obfuscation::deobfuscate,
	enums::{HeaderType, PlatformType},
	error::{VromfError, VromfError::IndexingFileOutOfBounds},
	util::{bytes_to_int, pack_type_from_aligned},
};

pub fn decode_bin_vromf(file: &[u8]) -> Result<Vec<u8>, VromfError> {
	let mut ptr = 0_usize;

	// Returns slice offset from file, incrementing the ptr by offset
	let idx_file_offset = |ptr: &mut usize, offset: usize| {
		if let Some(buff) = file.get(*ptr..(*ptr + offset)) {
			*ptr += offset;
			Ok(buff)
		} else {
			return Err(IndexingFileOutOfBounds {
				current_ptr:   *ptr,
				file_size:     file.len(),
				requested_len: offset,
			});
		}
	};

	let header_type = bytes_to_int(idx_file_offset(&mut ptr, 4)?)?;
	let header_type = HeaderType::try_from(header_type)?;

	let platform_raw = bytes_to_int(idx_file_offset(&mut ptr, 4)?)?;
	let _platform = PlatformType::try_from(platform_raw)?;

	let size = bytes_to_int(idx_file_offset(&mut ptr, 4)?)?;

	let header_packed: u32 = bytes_to_int(idx_file_offset(&mut ptr, 4)?)?;
	let (pack_type, extended_header_size) = pack_type_from_aligned(header_packed).unwrap();

	let inner_data = if header_type.is_extended() {
		let extended_header = idx_file_offset(
			&mut ptr,
			size_of::<u16>() + size_of::<u16>() + size_of::<u32>(),
		)?;
		let s = extended_header; // Copying ptr such that indexing below is less verbose

		// Unused header elements, for now
		let _header_size = u16::from_le_bytes([s[0], s[1]]);
		let _flags = u16::from_le_bytes([s[2], s[3]]);
		// The version is always reversed in order. It may never exceed 255
		let _version = (s[7], s[6], s[5], s[4]);

		idx_file_offset(&mut ptr, extended_header_size as usize)?
	} else {
		idx_file_offset(&mut ptr, size as usize)?
	};

	// Directly return when data is not obfuscated
	if !pack_type.is_obfuscated() {
		return Ok(inner_data.to_vec());
	}

	let mut output = inner_data.to_vec();
	deobfuscate(&mut output);

	if pack_type.is_compressed() {
		output = zstd::decode_all(output.as_slice()).unwrap();
	}

	Ok(output)
}

#[cfg(test)]
mod test {
	use std::fs;

	use crate::vromf::binary_container::decode_bin_vromf;

	#[test]
	fn decode_simple() {
		let f = fs::read("./samples/checked_simple_uncompressed_checked.vromfs.bin").unwrap();
		decode_bin_vromf(&f).unwrap();
	}

	#[test]
	fn decode_compressed() {
		let f = fs::read("./samples/unchecked_extended_compressed_checked.vromfs.bin").unwrap();
		decode_bin_vromf(&f).unwrap();
	}
}
