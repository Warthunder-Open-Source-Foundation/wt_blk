//! # Binary BLK
//! This module describes the binary BLK format and all of its know features and flags.
//! > ⚠ **Warning: Unless stated otherwise, all integers are Little-endian**
//!
//! ## Terminology
//! For ease of writing and reading, there are a few terms i will establish for this chapter
//! - u32 -> 32-bit unsigned integer
//! - u64 -> 64-bit unsigned integer
//! - f32 -> 32 bit float
//! - f64 -> 64 bit float
//! - \[T; N\] -> N long array of T
//! - offset -> absolute offset from 0 in the file
//! - nm -> A file containing Strings that is not in the BLK binary itself
//! - dict -> A ZSTD dictionary used in batch compressing many BLK files
//! - ULEB -> Unsigned LEB encoded variable length integer format: `https://en.wikipedia.org/wiki/LEB128`
//!
//! ## Types
//! BLK has 12 data-types
//!
//! |Name|Byte identifier|Layout|Size in bytes|[Inline](#inlining)|
//! |-|-|-|-|-|
//! |String|0x01|Zero terminated string|Variable||
//! |Int|0x02|i32|4|yes|
//! |Float|0x03|f32|4|yes|
//! |Float2|0x04|\[f32; 2\]|8||
//! |Float3|0x05|\[f32; 3\]|12||
//! |Float4|0x06|\[f32; 4\]|16||
//! |Int2|0x07|\[i32; 2\]|8||
//! |Int3|0x08|\[i32; 3\]|12||
//! |Bool|0x09|boolean|4|yes|
//! |Color|0x0a|\[u8; 4\]|4|yes|
//! |Float12|0x0b|\[f32; 12\]|48||
//! |Long|0x0c|i64|8||
//!
//! ### Inlining
//! When a type is inline, it means that its offset field contains the types data (used for small types 32 bits or smaller).  
//! Otherwise, the type data is an offset where the actual payload can be found.  
//!
//! ## Kinds of BLK files
//! There is not just one type of BLK, there are some important differences to denote.
//! |Byte ID|String ID|Description|
//! |-|-|-|
//! |0x00|BBF|A legacy format that this library does not understand|
//! |0x01|FAT|A standalone BLK binary, about as normal as it gets|
//! |0x02|FAT_ZST|Same as FAT, but ZSTD compressed|
//! |0x03|SLIM|A BLK like FAT, but with all strings outlined to the nm|
//! |0x04|SLIM_ZST|Same as SLIM, but ZSTD compressed|
//! |0x05|SLIM_ZSTD_DICT|Same as SLIM, but ZSTD compressed using the dict|
//! 
//! ## File layout
//! Now that we understand all necessary terms and types for BLK, were almost ready to encode/decode them.  
//! I will first explain the layout of a regular FAT file, then elaborate on the differences to the other kinds.
//! # Illustration of the fat BLK
//! ![Illustration](https://raw.githubusercontent.com/Warthunder-Open-Source-Foundation/wt_blk/refs/heads/master/charts/rendered/fat_blk_layout.png)
use std::{
	fs,
	fs::ReadDir,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Arc,
	},
};

pub use ::zstd::dict::DecoderDictionary;
use cfg_if::cfg_if;
use color_eyre::Report;
use blk_string::blk_str;
use crate::blk::{
	binary_deserialize::parser::parse_blk,
	blk_structure::BlkField,
	blk_type::BlkType,
	file::FileType,
	name_map::NameMap,
	zstd::decode_zstd,
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
pub mod name_map;

cfg_if! {
	if #[cfg(test)] {
		/// Unit tests
		pub mod test;
	}
}

/// Collection of macros and functions used in all BLK modules
pub mod util;

/// Zstandard unpacking functionality
pub mod zstd;

/// Implementations for serializing into human readable text formats from internal representation
pub mod plaintext_serialize;

/// Implementations for deserializing into internal representation format from text
mod plaintext_deserialize;

/// Implementation for deserializing internal representation to binary form
pub mod binary_deserialize;
mod blk_string;

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

/// Highest-level function for unpacking one BLK explicitly, for direct low level control call [`binary_deserialize::parser::parse_blk`]
pub fn unpack_blk(
	file: &mut Vec<u8>,
	dictionary: Option<&DecoderDictionary>,
	nm: Option<Arc<NameMap>>,
) -> Result<BlkField, Report> {
	let mut offset = 0;
	let file_type = FileType::from_byte(file[0])?;
	if file_type.is_zstd() {
		if file_type == FileType::FAT_ZSTD {
			offset += 1
		}; // FAT_ZSTD has a leading byte indicating that its unpacked form is of the FAT format
		*file = decode_zstd(file_type, &file, dictionary)?;
	} else {
		// uncompressed Slim and Fat files retain their initial magic bytes
		offset = 1;
	};

	let parsed = parse_blk(&file[offset..], file_type.is_slim(), nm)?;
	Ok(parsed)
}

pub fn make_strict_test() -> BlkField {
	BlkField::Struct(
		blk_str("root"),
		vec![
			BlkField::Value(
				blk_str("vec4f"),
				BlkType::Float4(Box::new([1.25, 2.5, 5.0, 10.0])),
			),
			BlkField::Value(blk_str("int"), BlkType::Int(42)),
			BlkField::Value(blk_str("long"), BlkType::Long(64)),
			BlkField::Struct(
				blk_str("alpha"),
				vec![
					BlkField::Value(blk_str("str"), BlkType::Str(blk_str("hello"))),
					BlkField::Value(blk_str("bool"), BlkType::Bool(true)),
					BlkField::Value(
						blk_str("color"),
						BlkType::Color {
							r: 3,
							g: 2,
							b: 1,
							a: 4,
						},
					),
					BlkField::Struct(
						blk_str("gamma"),
						vec![
							BlkField::Value(blk_str("vec2i"), BlkType::Int2([3, 4])),
							BlkField::Value(blk_str("vec2f"), BlkType::Float2([1.25, 2.5])),
							BlkField::Value(
								blk_str("transform"),
								BlkType::Float12(Box::new([
									1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.25, 2.5, 5.0,
								])),
							),
						],
					),
				],
			),
			BlkField::Struct(
				blk_str("beta"),
				vec![
					BlkField::Value(blk_str("float"), BlkType::Float(1.25)),
					BlkField::Value(blk_str("vec2i"), BlkType::Int2([1, 2])),
					BlkField::Value(blk_str("vec3f"), BlkType::Float3([1.25, 2.5, 5.0])),
				],
			),
		],
	)
}
