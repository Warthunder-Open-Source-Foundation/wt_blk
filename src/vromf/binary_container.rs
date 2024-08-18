use std::mem::size_of;

use color_eyre::{eyre::bail, Report, Section};
use wt_version::Version;

use crate::vromf::{
	de_obfuscation::deobfuscate,
	enums::{HeaderType, PlatformType},
	header::Metadata,
	util::{bytes_to_int, pack_type_from_aligned},
};

pub(crate) fn decode_bin_vromf(file: &[u8], validate: bool) -> Result<(Vec<u8>, Metadata), Report> {
	let mut metadata = Metadata::default();

	let mut ptr = 0_usize;

	// Returns slice offset from file, incrementing the ptr by offset
	let idx_file_offset = |ptr: &mut usize, offset: usize| {
		if let Some(buff) = file.get(*ptr..(*ptr + offset)) {
			*ptr += offset;
			Ok(buff)
		} else {
			Err(Report::msg(format!(
				"Indexing buffer of size {} with index {} and length {}",
				file.len(),
				*ptr,
				offset
			)))
		}
	};

	let header_type = bytes_to_int(idx_file_offset(&mut ptr, 4)?)?;
	let header_type = HeaderType::try_from(header_type)?;
	metadata.header_type = Some(header_type);

	let platform_raw = bytes_to_int(idx_file_offset(&mut ptr, 4)?)?;
	let platform = PlatformType::try_from(platform_raw)?;
	metadata.platform = Some(platform);

	// Size of the file before compression
	let size = bytes_to_int(idx_file_offset(&mut ptr, 4)?)?;

	let header_packed: u32 = bytes_to_int(idx_file_offset(&mut ptr, 4)?)?;

	// Type of compression/packing, and size before compression
	let (pack_type, extended_header_size) = pack_type_from_aligned(header_packed)?;
	metadata.packing = Some(pack_type);

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
		let version = [s[7], s[6], s[5], s[4]];
		metadata.version = Some(Version::new(
			version[0] as u16,
			version[1] as u16,
			version[2] as u16,
			version[3] as u16,
		));

		// Null length means the remaining bytes are used
		if extended_header_size == 0 {
			&file[ptr..]
		} else {
			idx_file_offset(&mut ptr, extended_header_size as usize)?
		}
	} else {
		if pack_type.is_compressed() {
			idx_file_offset(&mut ptr, extended_header_size as usize)?
		} else {
			idx_file_offset(&mut ptr, size as usize)?
		}
	};

	// Directly return when data is not obfuscated
	if !pack_type.is_obfuscated() {
		return Ok((inner_data.to_vec(), metadata));
	}

	let mut output = inner_data.to_vec();
	deobfuscate(&mut output);

	if pack_type.is_compressed() {
		output = zstd::decode_all(output.as_slice())
			.note("This most likely occurred because of improper computation of the frame-size")?;
	}

	if pack_type.has_hash() && validate {
		let expected = idx_file_offset(&mut ptr, 16)?;
		let computed_hash = md5::compute(&output);
		if expected != &computed_hash.0 {
			bail!(
				"Hash missmatch! Expected {:x} but found {computed_hash:x}",
				u128::from_le_bytes(expected.try_into()?)
			);
		}
	}

	Ok((output, metadata))
}

pub(crate) fn _encode_bin_vromf(_input: &[u8], _meta: Metadata) -> Result<Vec<u8>, Report> {
	todo!()
}

#[cfg(test)]
mod test {
	use std::fs;

	use crate::vromf::binary_container::decode_bin_vromf;

	#[test]
	fn decode_compressed() {
		let f = fs::read("./samples/unchecked_extended_compressed_checked.vromfs.bin").unwrap();
		decode_bin_vromf(&f, true).unwrap();
	}

	// #[test]
	// fn two_way() {
	// 	let f = fs::read("./samples/unchecked_extended_compressed_checked.vromfs.bin").unwrap();
	// 	let (decoded, meta) = decode_bin_vromf(&f).unwrap();
	// 	let re_encoded = encode_bin_vromf(&decoded, meta).unwrap();
	// 	assert_eq!(re_encoded, f);
	// }

	// #[test]
	// fn test_regional() {
	// 	let f = fs::read("./samples/regional.vromfs.bin").unwrap();
	// 	// decode_bin_vromf(&f).unwrap();
	// 	let unpacker = VromfUnpacker::from_file((PathBuf::from_str("asas").unwrap(), f)).unwrap();
	// 	unpacker.unpack_all(None).unwrap();
	// }
}
