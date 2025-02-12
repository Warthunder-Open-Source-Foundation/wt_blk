//! # Binary Container
//! This module describes the binary container and all known features and flags.
//! > ⚠ **Warning: Unless stated otherwise, all integers are Little-endian**
//! ## Header fields and their purpose
//!
//!	### Header type `32 bits`
//!
//! The base header is 16 bytes long, the extended version adds 8 additional bytes of information following the base.
//!
//! | Hex u32 | String repr. | Name | Meaning|
//! |---------|--------------|------|--------|
//! |0x73465256|`VRFs`       |VRFS  |File has a simple header|
//! |0x78465256|`VRFx`       |VRFX  |File has an extended header|
//!
//!	### Platform type `32 bits`
//! Describes the platform the file is intended for, appears to serve no significant purpose.
//! | Hex u32   | String repr.| Platform |
//! |-----------|-------------|----------|
//! |0x43500000 |`\0\0PC`     |PC        |
//! |0x534f6600 |`\0iOS`      |IOS       |
//! |0x646e06100|`\0and`      |Android   |
//!
//!	### Compression format `6 bits`
//! Defines the format of the inner payload.
//! | Hex   | Name              | Description of binary payload           |
//! |-------|-------------------|-----------------------------------------|
//! |0x30   |`ZSTD_OBFS`        |ZSTD compressed with obfuscation applied |
//! |0x10   |`ZSTD_OBFS_NOCHECK`|Same as `ZSTD_OBFS` without checksum     |
//! |0x20   |`PLAIN`            |Uncompressed                             |
//!
//! ### Compression info `32 bits`
//! The first 6 bits are the [Compression format](#compression-format-6-bits), the trailing bits are the size of the decompressed payload in bytes.
//! The size is 0 when no compression is used.
//! | Compression format| Size  |
//! |-------------------|-------|
//! |31..26             | 25..0 |
//!
//! ## Composition of the base header `128 bits - 16 bytes`
//!
//! |[Header type](#header-type-32-bits)| [Platform](#platform-type-32-bits) | Size of the payload in bytes | [Compression info](#compression-info-32-bits) |
//! |-|-|-|-|
//!
//! ## Composition of the extended header `64 bits - 8 bytes`
//!
//! |Size of the extended header in bytes|Flags |Game version|
//! |-|-|-|
//! |Always 8, no other size has been observed|Unknown purpose|Encoded in reverse|
//! |2 bytes|2 bytes|4 bytes|
//!
//! The version 1.2.3.4 would be encoded as `0x04 0x03 0x02 0x01`
//!
//! ## Checksum
//! The vromf may also contain a 16 byte checksum as indicated by the [Compression format](#compression-format-6-bits).
//! The algorithm used is MD5, and only applies to the uncompressed payload.
//! `ZSTD_OBFS_NOCHECK` does not have this checksum, the other two formats do.
//!
//! # Payload de-obfuscation
//! Due to unknown reasons, the payload is obfuscated with arbitrary bits.
//! Refer to [`crate::vromf::de_obfuscation::deobfuscate`] for solving this issue.
//!
//! # Layout of a binary container with simple header
//! ![Simple header](https://raw.githubusercontent.com/Warthunder-Open-Source-Foundation/wt_blk/refs/heads/master/charts/rendered/binary_container_with_simple_header.png)
//!
//! # Layout of a binary container with extended header
//! ![Extended header](https://raw.githubusercontent.com/Warthunder-Open-Source-Foundation/wt_blk/refs/heads/master/charts/rendered/binary_container_with_extended_header.png)
//!
//! # Continue reading
//! To learn how the payload is used, read [`crate::vromf::inner_container`].

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
		let header_size = u16::from_le_bytes([s[0], s[1]]);
		// No known case where the header is not 8 bytes.
		assert_eq!(header_size, 8, "Extended header size should be 8 bytes long");
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
