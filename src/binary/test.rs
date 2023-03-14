use std::fs;
use std::fs::ReadDir;
use std::io::{stdout, Write};
use std::process::exit;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use crate::binary::blk_structure::BlkField;
use crate::binary::blk_type::BlkCow;

use crate::binary::file::FileType;
use crate::binary::parser::parse_blk;
use crate::binary::zstd::{BlkDecoder, decode_zstd};
use crate::util::time;

#[cfg(test)]
mod test {
	use std::fs;
	use std::fs::ReadDir;
	use std::mem::size_of;
	use std::path::Path;
	use std::rc::Rc;
	use std::sync::{Arc, Mutex};
	use std::sync::atomic::{AtomicUsize, Ordering};
	use std::time::Instant;
	use zstd::Decoder;
	use zstd::dict::DecoderDictionary;

	use crate::binary::blk_type::BlkType;
	use crate::binary::file::FileType;
	use crate::binary::leb128::uleb128;
	use crate::binary::nm_file::{NameMap, parse_slim_nm};
	use crate::binary::parser::parse_blk;
	use crate::binary::test::{parse_file, test_parse_dir};
	use crate::binary::zstd::{BlkDecoder, decode_zstd};


	#[test]
	fn fat_blk() {
		let file = fs::read("./samples/section_fat.blk").unwrap();
		let output = parse_blk(&file, true, false, None, Rc::new(vec![]));
	}

	#[test]
	fn fat_blk_router_probe() {
		let file = fs::read("./samples/route_prober.blk").unwrap();
		let output = parse_blk(&file, false, false, None, Rc::new(vec![]));
	}

	/// the rendist file is *very* large for a BLK file, so this test is best for optimizing single-run executions
	#[test]
	fn slim_zstd_rendist() {
		let mut file = fs::read("./samples/rendist/rendinst_dmg.blk").unwrap();

		let nm = fs::read("./samples/rendist/nm").unwrap();
		let dict = fs::read("./samples/rendist/ca35013aabca60792d5203b0137d0a8720d1dc151897eb856b12318891d08466.dict").unwrap();

		let mut frame_decoder = DecoderDictionary::copy(&dict);

		let nm = NameMap::decode_nm_file(&nm).unwrap();
		let parsed_nm = parse_slim_nm(&nm);

		let mut offset = 0;
		let file_type = FileType::from_byte(file[0]).unwrap();
		if file_type.is_zstd() {
			file = decode_zstd(&file, Arc::new(frame_decoder)).unwrap();
		} else {
			// uncompressed Slim and Fat files retain their initial magic bytes
			offset = 1;
		}

		parse_blk(&file[offset..], false, file_type.is_slim(), Some(&nm), Rc::new(parsed_nm));
	}

	#[test]
	fn slim_blk() {
		let file = fs::read("./samples/section_slim.blk").unwrap();
		let nm = fs::read("./samples/names").unwrap();
		let parsed_nm = parse_slim_nm(&nm);
		let output = parse_blk(&file, true, true, Some(&nm), Rc::new(parsed_nm));
	}

	#[test]
	fn test_all() {
		let start = Instant::now();
		let nm = fs::read("./samples/vromfs/aces.vromfs.bin_u/nm").unwrap();
		let dict = fs::read("./samples/vromfs/aces.vromfs.bin_u/ca35013aabca60792d5203b0137d0a8720d1dc151897eb856b12318891d08466.dict").unwrap();

		let frame_decoder = DecoderDictionary::copy(&dict);

		let nm = NameMap::decode_nm_file(&nm).unwrap();
		let parsed_nm = parse_slim_nm(&nm);

		let dir: ReadDir = fs::read_dir("./samples/vromfs/aces.vromfs.bin_u").unwrap();
		let mut total = AtomicUsize::new(0);

		let mut pile = vec![];
		test_parse_dir(&mut pile, dir, &total);

		let rc_nm = Rc::new(parsed_nm);
		let arced_fd = Arc::new(frame_decoder);
		let out = pile.into_iter().map(|file| {
			parse_file(file.1, arced_fd.clone(), &nm, rc_nm.clone())
		}).filter_map(|x| x)
					  .collect::<Vec<_>>();

		let stop = start.elapsed();
		println!("Successfully parsed {} files! Thats all of them. The process took: {stop:?}", out.len());
	}
}

pub fn test_parse_dir(pile: &mut Vec<(String, Vec<u8>)>, dir: ReadDir, total_files_processed: &AtomicUsize) {
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

pub fn parse_file(mut file: Vec<u8>, fd: Arc<BlkDecoder>, nm: &[u8], parsed_nm: Rc<Vec<BlkCow>>) -> Option<String> {
	let mut offset = 0;
	let file_type = FileType::from_byte(file[0])?;
	if file_type.is_zstd() {
		file = decode_zstd(&file, fd.clone()).unwrap();
	} else {
		// uncompressed Slim and Fat files retain their initial magic bytes
		offset = 1;
	};


	Some(serde_json::to_string(&parse_blk(&file[offset..], false, file_type.is_slim(), Some(nm), parsed_nm.clone())).unwrap())
}