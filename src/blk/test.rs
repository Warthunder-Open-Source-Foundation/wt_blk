use std::{fs, sync::Arc, time::Instant};

use zstd::dict::DecoderDictionary;

use crate::blk::{
	binary_deserialize::parser::parse_blk,
	blk_string::blk_str,
	blk_structure::BlkField,
	blk_type::BlkType,
	file::FileType,
	make_strict_test,
	name_map::NameMap,
	unpack_blk,
	zstd::decode_zstd,
};

// #[test]
// fn json_parity() {
// 	let nm = fs::read("./samples/rendist/nm").unwrap();
// 	let dict = fs::read("./samples/rendist/ca35013aabca60792d5203b0137d0a8720d1dc151897eb856b12318891d08466.dict").unwrap();
// 	let frame_decoder = DecoderDictionary::copy(&dict);
//
// 	let mut file = fs::read("./samples/su_r_27er.blk").unwrap();
// 	file = decode_zstd(&file, Arc::new(frame_decoder)).unwrap();
// 	let shared_name_map = NameMap::from_encoded_file(&nm).unwrap();
// 	let output = parse_blk(&file, true, Arc::new(shared_name_map));
// 	assert_eq!(
// 		output
// 			.unwrap()
// 			.as_ref_json(FormattingConfiguration::GSZABI_REPO),
// 		fs::read_to_string("./samples/su_r_27er.blkx").unwrap()
// 	)
// }

#[test]
fn fat_blk() {
	let file = fs::read("./samples/section_fat.blk").unwrap();
	let output = parse_blk(&file[1..], false, None).unwrap();
	let expected = make_strict_test();
	assert_eq!(expected, output)
}

#[test]
fn netfile() {
	let file = fs::read("./samples/encoded_11.blk").unwrap();
	let output = parse_blk(&file[1..], false, None).unwrap();
	assert_eq!(
		output
			.pointer("slogan")
			.unwrap()
			.value()
			.unwrap()
			.to_string_default(),
		"t = \"○ Warriors at the gate ○\""
	)
}

#[test]
fn fat_blk_router_probe() {
	let file = fs::read("./samples/route_prober.blk").unwrap();
	let _output = parse_blk(&file, false, None).unwrap();
}

/// the rendist file is *very* large for a BLK file, so this test is best for optimizing single-run executions
#[test]
fn slim_zstd_rendist() {
	let mut file = fs::read("./samples/rendist/rendinst_dmg.blk").unwrap();

	let nm = fs::read("./samples/rendist/nm").unwrap();
	let dict = fs::read(
		"./samples/rendist/ca35013aabca60792d5203b0137d0a8720d1dc151897eb856b12318891d08466.dict",
	)
	.unwrap();

	let frame_decoder = DecoderDictionary::copy(&dict);

	let mut offset = 0;
	let file_type = FileType::from_byte(file[0]).unwrap();
	if file_type.is_zstd() {
		file = decode_zstd(file_type, &file, Some(&frame_decoder)).unwrap();
	} else {
		// uncompressed Slim and Fat files retain their initial magic bytes
		offset = 1;
	}

	let shared_name_map = NameMap::from_encoded_file(&nm).unwrap();
	let parsed = parse_blk(
		&file[offset..],
		file_type.is_slim(),
		Some(Arc::new(shared_name_map)),
	)
	.unwrap();

	assert_eq!(
		fs::read_to_string("./samples/rendist_sample_stability.txt").unwrap(),
		serde_json::to_string_pretty(&parsed).unwrap()
	);
}

#[test]
fn slim_blk() {
	let file = fs::read("./samples/section_slim.blk").unwrap();
	let nm = fs::read("./samples/nm").unwrap();

	let shared_name_map = NameMap::from_encoded_file(&nm).unwrap();
	let start = Instant::now();
	let output = parse_blk(&file[1..], true, Some(Arc::new(shared_name_map))).unwrap();
	println!("{:?}", start.elapsed());
	println!("{:?}", output.estimate_size());
}

#[test]
fn slim_blk_with_its_own_names() {
	let shared = NameMap {
		binary: vec![],
		parsed: Arc::new(vec![blk_str("alpha"), blk_str("beta")]),
	};
	let mut dump = vec![1, 6]; // 1 own name, 6 bytes
	dump.extend_from_slice(b"gamma\0");
	dump.extend_from_slice(&[1, 2, 6]); // 1 block, 2 params, 6 bytes of data
	dump.extend_from_slice(b"hello\0");
	// "gamma" (own name 2, after the 2 shared ones) is a string from this file's data
	dump.extend_from_slice(&[2, 0, 0, 1]);
	dump.extend_from_slice(&0u32.to_le_bytes());
	// "alpha" is a string naming entry 2 of the name map
	dump.extend_from_slice(&[0, 0, 0, 1]);
	dump.extend_from_slice(&(2u32 | 1 << 31).to_le_bytes());
	dump.extend_from_slice(&[0, 2, 0]); // root: 2 params, no blocks

	let parsed = parse_blk(&dump, true, Some(Arc::new(shared))).unwrap();
	assert_eq!(
		parsed,
		BlkField::Struct(
			blk_str("root"),
			vec![
				BlkField::Value(blk_str("gamma"), BlkType::Str(blk_str("hello"))),
				BlkField::Value(blk_str("alpha"), BlkType::Str(blk_str("gamma"))),
			]
		)
	);
}

#[test]
fn blk_without_names() {
	// No names means no size for them either
	let dump = [0, 1, 0, 0, 0, 0, 0];
	let parsed = parse_blk(&dump, false, None).unwrap();
	assert_eq!(parsed, BlkField::Struct(blk_str("root"), vec![]));
}

#[test]
fn empty_blk_is_an_error() {
	assert!(unpack_blk(&mut vec![], None, None).is_err());
}
