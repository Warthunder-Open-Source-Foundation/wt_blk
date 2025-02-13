//! # Inner container
//! This module describes the inner container and all of its know features and flags.
//! > ⚠ **Warning: Unless stated otherwise, all integers are Little-endian**
//!
//! ## Terminology
//! For ease of writing and reading, there are a few terms i will establish for this chapter
//! - "Names" refers to the strings used for the absolute path in the VROMF image
//! - "Data" refers to the payload of each file
//!
//! In combination, Names and data creates the file that you normally find in a folder once unpacked
//!
//! - u32 -> 32-bit unsigned integer
//! - u64 -> 64-bit unsigned integer
//! - offset -> absolute offset from 0 in the inner container
//!
//! ## Header fields and their purpose
//!
//! #### Names header `128 bits`
//! Offset points to the start of the [names info array](#names-info), count defines how many elements there are<br/>
//! The first byte also indicates the following:
//! - 0x20 -> no individual file checksums
//! - 0x30 -> individual file checksums **and the checksum `Begin` is not 0x00**¹
//!
//! |Offset |Count  |Padding |
//! |-|-|-|
//! |u32|u32|u64|
//!
//! ¹[checksum header](#checksum-header-128-bits)
//! ---
//!
//! #### Data header `128 bits`
//! Offset points to the start of the [data info array](#names-info), count defines how many elements there are
//! |Offset |Count |Padding |
//! |-|-|-|
//! |u32|u32|u64|
//! ---
//!
//! #### Checksum header `128 bits`
//! Marks the start and ending region of the checksums, one checksum for each payload.
//!
//! 20 byte long sha1 hashes, where each N-th multiple of 20 corresponds to the N-th data block<br/>
//! ⚠ **Warning: When the `Begin` field is 0x00, it means only the binary container has a checksum, not each individual file though**
//! |End |Begin |
//! |-|-|
//! |u64|u64|
//! ---
//!
//! #### Names Info
//! An array of offsets to the start of the N-th Name, with their end being a null byte
//! |Offset |
//! |-|
//! |u32|
//! ---
//!
//! #### Data Info
//! An array of offsets and length pairs pointing to the start of the N-th payload
//! |Offset |Length |Padding |
//! |-|-|-|
//! |u32|u32|u64|
//! ---


use std::{mem::size_of, path::PathBuf};
use std::io::Write;
use std::path::Path;
use color_eyre::{
	eyre::{bail, Context, ContextCompat},
	Report,
};
use fallible_iterator::{convert, FallibleIterator};
use sha1_smol::Sha1;

use crate::{
	util::join_hex,
	vromf::{
		util::{bytes_to_int, bytes_to_usize},
		File,
	},
};
use crate::repacker_util::Buffer;

pub fn decode_inner_vromf(file: &[u8], validate: bool) -> Result<Vec<File>, Report> {
	// Returns slice offset from file, incrementing the ptr by offset
	let idx_file_offset = |ptr: &mut usize, offset: usize| {
		if let Some(res) = file.get(*ptr..(*ptr + offset)) {
			*ptr += offset;
			Ok(res)
		} else {
			Err(Report::msg(format!(
				"Indexing buffer of size {} with index {} and length {}",
				file.len(),
				*ptr,
				offset
			)))
		}
	};
	let mut ptr = 0;

	// The header indicates existence of a digest
	let names_header = idx_file_offset(&mut ptr, size_of::<u32>())?;
	let mut has_digest = match names_header[0] {
		0x20 => false,
		0x30 => true,
		_ => {
			bail!("Unknown digest header {:X}", names_header[0])
		},
	};

	let names_offset = bytes_to_int(names_header)? as usize;
	let names_count = bytes_to_int(idx_file_offset(&mut ptr, size_of::<u32>())?)? as usize;
	ptr += size_of::<u32>() * 2; // Padding to 16 byte alignment

	let data_info_offset = bytes_to_int(idx_file_offset(&mut ptr, size_of::<u32>())?)? as usize;
	let data_info_count = bytes_to_int(idx_file_offset(&mut ptr, size_of::<u32>())?)? as usize;
	ptr += size_of::<u32>() * 2; // Padding to 16 byte alignment

	let mut digest_data = if has_digest {
		let digest_end = bytes_to_usize(idx_file_offset(&mut ptr, size_of::<u64>())?)?;
		let digest_begin = bytes_to_usize(idx_file_offset(&mut ptr, size_of::<u64>())?)?;
		// Special case; The VROMF has a hash over the entire container but not individual files
		if digest_begin == 0 {
			has_digest = false;
		}
		let digest_data = &file[digest_begin..digest_end];
		let chunks = digest_data.chunks_exact(20);
		Some(chunks)
	} else {
		None
	};

	// Names info is a set of u64s, pointing at each name
	let names_info_len = names_count * size_of::<u64>();
	let names_info = &file[names_offset..(names_offset + names_info_len)];
	let names_info_chunks = names_info.array_chunks::<{ size_of::<u64>() }>(); // No remainder from chunks as it is infallible
	let parsed_names_offsets: Vec<usize> = names_info_chunks
		.into_iter()
		.map(|x| bytes_to_usize(x))
		.collect::<Result<_, Report>>()?;
	let file_names = parsed_names_offsets.into_iter().map(|start| {
		let mut buff = Vec::with_capacity(32);
		for byte in &file[start..] {
			if *byte == 0 {
				break;
			} else {
				buff.push(*byte)
			}
		}
		// The nm file has a special case, where it has additional "garbage" bytes leading in-front of it
		const NM_BYTE_ID: &[u8] = b"\xff\x3fnm";
		if let Some(leading_bytes) = buff.get(..4) {
			if leading_bytes == NM_BYTE_ID {
				buff = b"nm".to_vec();
			}
		}
		let s = String::from_utf8(buff)
			.map(|res| PathBuf::from(res))
			.context("Invalid UTF-8 sequence".to_string())?;
		Ok::<PathBuf, Report>(s)
	});

	// FYI:
	// Each data-info-block consists of 4x u32
	// Only the first two values are used, as offset and length, the remaining two values are 0
	let data_info_len = data_info_count * size_of::<u32>() * 4; // Total length of the data-info block
	let data_info = &file[data_info_offset..(data_info_offset + data_info_len)];
	let data_info_split = data_info.array_chunks::<{ size_of::<u32>() }>(); // Data-info consists of u32 pairs, so we will split them once
	if data_info_split.remainder().len() != 0 {
		bail!("Unaligned chunks: the data-set of size {} was supposed to align/chunk into {}, but {} remained", data_info.len(), size_of::<u32>(), data_info_split.remainder().len());
	}

	// This has to align to 4, because of previous chunk checks
	let data_info_full = data_info_split.array_chunks::<4>(); // Join together pairs of offset and length with 2 trailing bytes
	let data = data_info_full
		.map(|x| {
			(
				u32::from_le_bytes(*x[0]) as usize,
				u32::from_le_bytes(*x[1]) as usize,
			)
		})
		.map(|(offset, size)| file[offset..(offset + size)].to_vec())
		.map(|e| {
			// Check digest only if the file should have one
			if validate && has_digest {
				let digest = digest_data
					.as_mut()
					.map(|e| e.next())
					.context("Digest missing")?
					.context("Too few digest elements")?;
				let h = Sha1::from(&e).digest().bytes();
				if digest != &h {
					println!(
						"Hash mismatch: expected: {} but found {}",
						join_hex(&digest),
						join_hex(&h)
					);
				}
			}
			Ok(e)
		});

	Ok(convert(file_names)
		.zip(convert(data))
		.map(|(p, f)| Ok(File::from_raw(p, f)))
		.collect()?)
}


pub fn encode_inner_vromf(files: Vec<File>, digest_header: u8) -> Result<Vec<u8>, Report> {
	let has_digest = match digest_header {
		0x20 => false,
		0x30 => true,
		_ => {
			bail!("Unknown digest header {:X}", digest_header)
		},
	};

	let mut buf = Buffer{ inner: Default::default() };

	// **Names header**
	let names_offset = buf.u32()?;
	names_offset.set_write(digest_header as _, &mut buf)?;
	let names_count = buf.u32()?;
	names_count.set_write(files.len().try_into()?, &mut buf)?;
	buf.align_to_multiple_of_16()?;


	// **Data header**
	let data_info_offset = buf.u32()?;
	let data_info_count = buf.u32()?;
	data_info_count.set_write(files.len().try_into()?, &mut buf)?;
	buf.align_to_multiple_of_16()?;


	// **Digest header**
	let digest_data = if has_digest {
		let digest_end = buf.u64()?;
		let digest_begin = buf.u64()?;
		Some((digest_begin, digest_end))
	} else {
		None
	};
	buf.align_to_multiple_of_16()?;

	// **Names Info**
	let mut names_offsets = Vec::with_capacity(files.len());
	for _ in &files {
		let offs = buf.u64()?;
		names_offsets.push(offs);
	}
	buf.align_to_multiple_of_16()?;

	// **names data**
	for (file, index) in files.iter().zip(names_offsets.into_iter()) {
		let start = buf.inner.position();
		if file.path() == Path::new("nm") {
			buf.inner.write_all(b"\xff\x3fnm")?;
		} else {
			buf.inner.write_all(file.path().to_str().unwrap().as_ref())?;
		}
		buf.inner.write_all(&[0; 1])?;
		index.set_write(start, &mut buf)?;
	}
	buf.align_to_multiple_of_16()?;


	// **Data Info**
	let mut data_offsets = Vec::with_capacity(files.len());
	data_info_offset.set_write(buf.inner.position().try_into()?, &mut buf)?;
	for _ in &files {
		let offs = buf.u32()?;
		let size = buf.u32()?;
		buf.pad_zeroes::<{size_of::<u32>() * 2}>()?;
		data_offsets.push((offs, size));
	}
	buf.align_to_multiple_of_16()?;

	// **Digest data**

	if has_digest {
		let (start, end) = digest_data.expect("Infallible");
		start.set_write(buf.inner.position().try_into()?, &mut buf)?;
		for file in &files {
			let h = Sha1::from(file.buf()).digest().bytes();
			buf.inner.write_all(&h)?
		}
		end.set_write(buf.inner.position().try_into()?, &mut buf)?;
		buf.align_to_multiple_of_16()?;
	}

	// **Data**
	for (file, (offset, size)) in files.iter().zip(data_offsets.into_iter()) {
		let start = buf.inner.position();
		buf.inner.write_all(file.buf())?;
		offset.set_write(start.try_into()?, &mut buf)?;
		size.set_write(file.buf().len().try_into()?, &mut buf)?;
		buf.align_to_multiple_of_16()?;
	}


	Ok(buf.inner.into_inner())
}


#[cfg(test)]
mod test {
	use std::fs;

	use crate::vromf::{binary_container::decode_bin_vromf, inner_container::decode_inner_vromf};
	use crate::vromf::inner_container::encode_inner_vromf;

	#[test]
	fn test_uncompressed() {
		let f = fs::read("./samples/checked_simple_uncompressed_checked.vromfs.bin").unwrap();
		let (decoded, _) = decode_bin_vromf(&f, true).unwrap();
		let _inner = decode_inner_vromf(&decoded, true).unwrap();
	}

	#[test]
	fn test_compressed() {
		let f = fs::read("./samples/unchecked_extended_compressed_checked.vromfs.bin").unwrap();
		let (decoded, _) = decode_bin_vromf(&f, true).unwrap();
		let _inner = decode_inner_vromf(&decoded, true).unwrap();
	}

	#[test]
	fn test_checked() {
		let f = fs::read("./samples/checked.vromfs").unwrap();
		let _inner = decode_inner_vromf(&f, true).unwrap();
	}

	#[test]
	fn test_aces() {
		let f = fs::read("./samples/aces.vromfs.bin").unwrap();
		let (decoded, _) = decode_bin_vromf(&f, true).unwrap();
		let _inner = decode_inner_vromf(&decoded, true).unwrap();
	}

	#[test]
	fn test_aces_repack() {
		let f = fs::read("./samples/aces.vromfs.bin").unwrap();
		let (decoded, _) = decode_bin_vromf(&f, true).unwrap();
		let inner = decode_inner_vromf(&decoded, true).unwrap();
		let re_encoded = encode_inner_vromf(inner, decoded[0]).unwrap();

		assert_eq!(re_encoded.len(), decoded.len());
		assert_eq!(re_encoded, decoded);
	}

	#[test]
	fn test_checked_repack() {
		let f = fs::read("./samples/checked.vromfs").unwrap();
		let inner = decode_inner_vromf(&f, true).unwrap();
		let re_encoded = encode_inner_vromf(inner.clone(), f[0]).unwrap();

		assert_eq!(re_encoded.len(), f.len());
		assert_eq!(re_encoded, f);
	}
}
