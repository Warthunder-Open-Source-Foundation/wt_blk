use std::fs;
use std::fs::ReadDir;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub use ::zstd::dict::DecoderDictionary;

use crate::binary::blk_type::BlkString;
use crate::binary::file::FileType;
use crate::binary::nm_file::NameMap;
use crate::binary::parser::parse_blk;
use crate::binary::zstd::{BlkDecoder, decode_zstd};

pub mod file;
pub mod test;
pub mod parser;
pub mod leb128;
pub mod zstd;
pub mod blk_type;
pub mod blk_structure;
pub mod nm_file;
mod blk_to_text;
mod blk_block_hierarchy;
mod blk_to_ref_json;
pub mod output_formatting_conf;
mod error;
pub mod util;

fn test_parse_dir(pile: &mut Vec<(String, Vec<u8>)>, dir: ReadDir, total_files_processed: &AtomicUsize) {
	for file in dir {
		let file = file.as_ref().unwrap();
		if file.metadata().unwrap().is_dir() {
			test_parse_dir(pile, file.path().read_dir().unwrap(), total_files_processed);
		} else {
			let fname = file.file_name().to_str().unwrap().to_owned();
			if fname.ends_with(".blk") {
				let mut read = fs::read(file.path()).unwrap();
				pile.push((fname, read));
				total_files_processed.fetch_add(1, Ordering::Relaxed);
			}
		}
	}
}

pub fn parse_file(mut file: Vec<u8>, fd: Arc<BlkDecoder>, shared_name_map: Arc<NameMap>) -> Option<String> {
	let mut offset = 0;
	let file_type = FileType::from_byte(file[0])?;
	if file_type.is_zstd() {
		file = decode_zstd(&file, fd.clone()).unwrap();
	} else {
		// uncompressed Slim and Fat files retain their initial magic bytes
		offset = 1;
	};


	Some(serde_json::to_string(&parse_blk(&file[offset..], file_type.is_slim(), shared_name_map).ok()?).unwrap())
}