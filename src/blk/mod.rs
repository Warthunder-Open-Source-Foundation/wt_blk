use std::{
	fs,
	fs::ReadDir,
	sync::{
		Arc,
		atomic::{AtomicUsize, Ordering},
	},
};

pub use ::zstd::dict::DecoderDictionary;

use crate::blk::{
	file::FileType,
	nm_file::NameMap,
	parser::parse_blk,
	zstd::{BlkDecoder, decode_zstd},
};

/// Decodes flat map of fields into the corresponding nested datastructure
mod blk_block_hierarchy;

/// Defines the recursive/nested structure that BLK files are represented with internally
pub mod blk_structure;

/// Defines the primitive types that BLK stores
pub mod blk_type;

/// Shared error that is returned from hot functions,
/// otherwise, [`color_eyre::Report`] is used
pub mod error;

/// One-byte file header that each blk file begins with
pub mod file;

/// Utility function to decode ULEB128 encoded files
/// <https://en.wikipedia.org/wiki/LEB128>
pub mod leb128;

/// Struct storing a shared map of strings that multiple BLK files reference
pub mod nm_file;

/// Exports core function for unpacking BLK file
pub mod parser;

/// Unit tests
#[cfg(test)]
pub mod test;

/// Collection of macros and functions used in all BLK modules
pub mod util;

/// Zstandard unpacking functionality
pub mod zstd;
mod repack;

/// Implementations for serializing into human readable text formats
pub mod plaintext_serialize;

/// Implementations for deserializing into BLk binary format from BlkText
mod plaintext_deserialize;

#[allow(dead_code)]
fn test_parse_dir(
	pile: &mut Vec<(String, Vec<u8>)>,
	dir: ReadDir,
	total_files_processed: &AtomicUsize,
) {
	for file in dir {
		let file = file.as_ref().unwrap();
		if file.metadata().unwrap().is_dir() {
			test_parse_dir(pile, file.path().read_dir().unwrap(), total_files_processed);
		} else {
			let fname = file.file_name().to_str().unwrap().to_owned();
			if fname.ends_with(".blk") {
				let read = fs::read(file.path()).unwrap();
				pile.push((fname, read));
				total_files_processed.fetch_add(1, Ordering::Relaxed);
			}
		}
	}
}

/// Highest-level function for unpacking one BLK explicitly, for direct low level control call [`parser::parse_blk`]
pub fn parse_file(
	mut file: Vec<u8>,
	fd: Arc<BlkDecoder>,
	shared_name_map: Option<Arc<NameMap>>,
) -> Option<String> {
	let mut offset = 0;
	let file_type = FileType::from_byte(file[0]).ok()?;
	if file_type.is_zstd() {
		file = decode_zstd(file_type, &file, Some(fd.as_ref())).unwrap();
	} else {
		// uncompressed Slim and Fat files retain their initial magic bytes
		offset = 1;
	};

	Some(
		serde_json::to_string(
			&parse_blk(&file[offset..], file_type.is_slim(), shared_name_map).ok()?,
		)
		.unwrap(),
	)
}
