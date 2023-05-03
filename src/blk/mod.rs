use std::{
	fs,
	fs::ReadDir,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Arc,
	},
};

pub use ::zstd::dict::DecoderDictionary;

use crate::blk::{
	file::FileType,
	nm_file::NameMap,
	parser::parse_blk,
	zstd::{decode_zstd, BlkDecoder},
};

mod blk_block_hierarchy;
pub mod blk_structure;
mod blk_to_ref_json;
mod blk_to_text;
pub mod blk_type;
pub mod error;
pub mod file;
pub mod leb128;
pub mod nm_file;
pub mod output_formatting_conf;
pub mod parser;
pub mod test;
pub mod util;
pub mod zstd;

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

pub fn parse_file(
	mut file: Vec<u8>,
	fd: Arc<BlkDecoder>,
	shared_name_map: Arc<Option<NameMap>>,
) -> Option<String> {
	let mut offset = 0;
	let file_type = FileType::from_byte(file[0]).ok()?;
	if file_type.is_zstd() {
		file = decode_zstd(&file, fd.clone()).unwrap();
	} else {
		// uncompressed Slim and Fat files retain their initial magic bytes
		offset = 1;
	};

	Some(
		serde_json::to_string(
			&parse_blk(&file[offset..], file_type.is_slim(),  shared_name_map).ok()?,
		)
		.unwrap(),
	)
}


pub enum BlkOutputFormat {
	Json,
	BlkText,
}