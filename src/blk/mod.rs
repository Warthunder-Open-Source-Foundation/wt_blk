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
//! - Name/Name map -> String as key or value in the BLK, map is an array of Strings
//!
//! ### Types
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
//! ### Kinds of BLK files
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
//! ### Name reference
//! Whenever a String is used in the name-map or field-map, this layout applies.
//! If the tag-bit is set, it means the index has to be looked up from the [external nm](#external-nm), otherwise the regular [nm](#name-map),
//! |Tag|Index|
//! |-|-|
//! |1 bit| 31 bit unsigned integer|
//!
//! ## File layout
//! Now that we understand all necessary terms and types for BLK, were almost ready to encode/decode them.
//! I will first explain the layout of a regular FAT file, then elaborate on the differences to the other kinds.
//! ![Illustration](https://raw.githubusercontent.com/Warthunder-Open-Source-Foundation/wt_blk/refs/heads/master/charts/rendered/fat_blk_layout.png)
//! The file starts off with a single byte describing its [kind](#kinds-of-blk-files), FAT in this case, of course.
//! After that, we begin the first section of data already called the [Name map].
//!
//! ### Name Map
//! Defines where any Strings used as keys or values in this BLK.
//! In the case of SLIM, only the names count ULEB will be present, skipping straight to the [struct count](#struct-count).
//! In the case of FAT, following the names count (ULEB) N is the names buffer size (ULEB) S.
//! After this comes all null seperated strings that should be as many as N specified.
//!
//! ### Block count
//! A single ULEB defining how many nested structs the BLK file contains (used later).
//!
//! ### Field map
//! Similar to the name map, here we first get the amount of fields followed by the size of the payload buffer.
//! The field payloads come before the field definitions and therefore need to be used during the decoding of the fields.
//! Each field definition is 32 bits in size structured as such:
//! |[Name ID](#name-reference)|Type ID|Offset|
//! |-|-|-|
//! |u24|u8|u32|
//! ---
//! When the type is inline, simply interpret the offset as its payload.
//! Otherwise, use the offset relative to the start of the payload buffer, reading as many bytes as the type needs.
//!
//! ### Nesting map
//! Until now, we have only read and parsed a list of fields. But you may ask, doesn't BLK support nested structs?
//! Indeed, it does, and it uses the following layout to figure out which fields and struct correspond to which parent struct.
//! The data layout is as such:
//! |Index|Struct name|field count|sub-structs count|sub-struct index (optional)|
//! |-|-|-|-|
//! |ULEB|[Name ID](#name-reference)|ULEB|ULEB|
//! ---
//! Explaining the algorithm necessary to structure this data is not trivial, so I will explain each value and its purpose with care.
//! However, I believe that, reading [`crate::blk::blk_block_hierarchy::FlatBlock`] (implementation) together with this text will work better than just the text.
//!
//! To start off, we first get the index, which uniquely identifies any struct, where 0 is the root/core struct.
//! Together with the index, we can determine the name, which is a [Name ID](#name-reference) or undefined and irrelevant if the index is 0.
//! 
//! Field `count` defines how many fields from [the field map](#field-map) belong to this struct in the order as they appear.
//! Keeping track of the sum of previous`count`'s is important as current `count` starts from where the last field ended. 
//! 
//! Sub-structs count defines how many substructs there are.
//! 
//! Sub-structs index defines which other struct are contained in this one, using the same indexing system as field count.
//! **This value is not present when sub-structs count was 0.**
//! 
//! This mechanism is best explained with an example to go with it:
//! Lets use this BLK as our working minimal example:
//! ```blk
//! "vec4f":p4 = 1.25, 2.5, 5, 10
//! "int":i = 42
//! "long":i64 = 0x40
//! "alpha" {
//!		"str":t = "hello"
//!		"bool":b = true
//!		"color":c = 0x1, 0x2, 0x3, 0x4
//!		"gamma" {
//!			"vec2i":ip2 = 3, 4
//!			"vec2f":p2 = 1.25, 2.5
//!			"transform":m = [[1, 0, 0] [0, 1, 0] [0, 0, 1] [1.25, 2.5, 5]]
//!		}
//! }
//! "beta" {
//!		"float":r = 1.25
//!		"vec2i":ip2 = 1, 2
//!		"vec3f":p3 = 1.25, 2.5, 5
//! }
//!```
//! The Nesting map would look like the following:
//! |Index|Name|Indexes|Sub-blocks|Binary representation|
//! |-|-|-|-|-|
//! |0|N.A.|0,1,2|1,2|`0x00 0x03 0x02 0x01`|
//! |1|alpha|3,4,5|3|`0x04 0x03 0x01 0x03` |
//! |2|beta|6,7,8||`0x0C 0x03 0x00` |
//! |3|gamma|9,10,11||`0x08 0x03 0x00` |

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

/// Decodes flat map of fields into the corresponding nested datastructures
pub(crate) mod blk_block_hierarchy;

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
