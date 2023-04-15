use std::fs;
use std::fs::ReadDir;
use std::io::{stdout, Write};
use std::process::exit;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;

use crate::blk::blk_structure::BlkField;
use crate::blk::blk_type::BlkString;
use crate::blk::file::FileType;
use crate::blk::parser::parse_blk;
use crate::blk::zstd::{BlkDecoder, decode_zstd};
use crate::util::time;

#[cfg(test)]
mod test {
	use std::fs;
	use std::fs::ReadDir;
	use std::mem::size_of;
	use std::path::Path;
	use std::process::exit;
	use std::rc::Rc;
	use std::sync::{Arc, Mutex};
	use std::sync::atomic::{AtomicUsize, Ordering};
	use std::time::Instant;

	use zstd::Decoder;
	use zstd::dict::DecoderDictionary;

	use crate::blk::{parse_file, test_parse_dir};
	use crate::blk::blk_type::BlkType;
	use crate::blk::file::FileType;
	use crate::blk::leb128::uleb128;
	use crate::blk::nm_file::NameMap;
	use crate::blk::output_formatting_conf::FormattingConfiguration;
	use crate::blk::parser::parse_blk;
	use crate::blk::zstd::{BlkDecoder, decode_zstd};

	#[test]
	fn json_parity() {
		let nm = fs::read("./samples/rendist/nm").unwrap();
		let dict = fs::read("./samples/rendist/ca35013aabca60792d5203b0137d0a8720d1dc151897eb856b12318891d08466.dict").unwrap();
		let mut frame_decoder = DecoderDictionary::copy(&dict);

		let mut file = fs::read("./samples/su_r_27er.blk").unwrap();
		file = decode_zstd(&file, Arc::new(frame_decoder)).unwrap();
		let shared_name_map = NameMap::from_encoded_file(&nm).unwrap();
		let output = parse_blk(&file, true, Arc::new(shared_name_map));
		assert_eq!(output.unwrap().as_ref_json(FormattingConfiguration::GSZABI_REPO), fs::read_to_string("./samples/su_r_27er.blkx").unwrap())
	}

	#[test]
	fn fat_blk() {
		let file = fs::read("./samples/section_fat.blk").unwrap();
		let output = parse_blk(&file[1..], false, NameMap::DUMMY());
		println!("{}", output.unwrap().as_blk_text());
	}

	#[test]
	fn fat_blk_router_probe() {
		let file = fs::read("./samples/route_prober.blk").unwrap();
		let output = parse_blk(&file, false, NameMap::DUMMY()).unwrap();
	}

	/// the rendist file is *very* large for a BLK file, so this test is best for optimizing single-run executions
	#[test]
	fn slim_zstd_rendist() {
		let mut file = fs::read("./samples/rendist/rendinst_dmg.blk").unwrap();

		let nm = fs::read("./samples/rendist/nm").unwrap();
		let dict = fs::read("./samples/rendist/ca35013aabca60792d5203b0137d0a8720d1dc151897eb856b12318891d08466.dict").unwrap();

		let mut frame_decoder = DecoderDictionary::copy(&dict);

		let mut offset = 0;
		let file_type = FileType::from_byte(file[0]).unwrap();
		if file_type.is_zstd() {
			file = decode_zstd(&file, Arc::new(frame_decoder)).unwrap();
		} else {
			// uncompressed Slim and Fat files retain their initial magic bytes
			offset = 1;
		}

		let shared_name_map = NameMap::from_encoded_file(&nm).unwrap();
		let parsed = parse_blk(&file[offset..], file_type.is_slim(), Arc::new(shared_name_map)).unwrap();

		assert_eq!(fs::read_to_string("./samples/rendist_sample_stability.txt").unwrap(), serde_json::to_string_pretty(&parsed).unwrap());
	}

	#[test]
	fn slim_blk() {
		let file = fs::read("./samples/section_slim.blk").unwrap();
		let nm = fs::read("./samples/nm").unwrap();

		let shared_name_map = NameMap::from_encoded_file(&nm).unwrap();
		let output = parse_blk(&file[1..], true, Arc::new(shared_name_map)).unwrap();
	}

	#[test]
	fn test_all() {
		let start = Instant::now();
		let nm = fs::read("./samples/vromfs/aces.vromfs.bin_u/nm").unwrap();
		let dict = fs::read("./samples/vromfs/aces.vromfs.bin_u/ca35013aabca60792d5203b0137d0a8720d1dc151897eb856b12318891d08466.dict").unwrap();

		let frame_decoder = DecoderDictionary::copy(&dict);


		let dir: ReadDir = fs::read_dir("./samples/vromfs/aces.vromfs.bin_u").unwrap();
		let mut total = AtomicUsize::new(0);

		let mut pile = vec![];
		test_parse_dir(&mut pile, dir, &total);

		let shared_name_map = Arc::new(NameMap::from_encoded_file(&nm).unwrap());
		let arced_fd = Arc::new(frame_decoder);
		let out = pile.into_iter().map(|file| {
			parse_file(file.1, arced_fd.clone(), shared_name_map.clone())
		}).filter_map(|x| x)
					  .collect::<Vec<_>>();

		let stop = start.elapsed();
		println!("Successfully parsed {} files! Thats all of them. The process took: {stop:?}", out.len());
	}
}