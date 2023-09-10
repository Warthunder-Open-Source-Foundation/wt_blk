use std::{
	fs,
	fs::ReadDir,
	sync::{atomic::AtomicUsize, Arc},
	time::Instant,
};

use zstd::dict::DecoderDictionary;

use crate::blk::{
	file::FileType,
	nm_file::NameMap,
	parse_file,
	parser::parse_blk,
	test_parse_dir,
	zstd::decode_zstd,
};
use crate::blk::blk_structure::BlkField;
use crate::blk::blk_type::BlkType;
use crate::blk::util::blk_str;

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
	let expected = BlkField::Struct(blk_str("root"), vec![
		BlkField::Value(blk_str("vec4f"), BlkType::Float4([1.25, 2.5, 5.0, 10.0])),
		BlkField::Value(blk_str("int"), BlkType::Int(42)),
		BlkField::Value(blk_str("long"), BlkType::Long(64)),
		BlkField::Struct(blk_str("alpha"), vec![
			BlkField::Value(blk_str("str"), BlkType::Str(blk_str("hello"))),
			BlkField::Value(blk_str("bool"), BlkType::Bool(true)),
			BlkField::Value(blk_str("color"), BlkType::Color { r: 3, g: 2, b: 1, a: 4 }),
			BlkField::Struct(blk_str("gamma"), vec![
				BlkField::Value(blk_str("vec2i"), BlkType::Int2([3, 4])),
				BlkField::Value(blk_str("vec2f"), BlkType::Float2([1.25, 2.5])),
				BlkField::Value(blk_str("transform"), BlkType::Float12(Box::new([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.25, 2.5, 5.0]))),
			]),
		]),
		BlkField::Struct(blk_str("beta"), vec![
			BlkField::Value(blk_str("float"), BlkType::Float(1.25)),
			BlkField::Value(blk_str("vec2i"), BlkType::Int2([1, 2])),
			BlkField::Value(blk_str("vec3f"), BlkType::Float3([1.25, 2.5, 5.0])),
		]),
	]);
	assert_eq!(expected, output)
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
	let dict = fs::read("./samples/rendist/ca35013aabca60792d5203b0137d0a8720d1dc151897eb856b12318891d08466.dict").unwrap();

	let frame_decoder = DecoderDictionary::copy(&dict);

	let mut offset = 0;
	let file_type = FileType::from_byte(file[0]).unwrap();
	if file_type.is_zstd() {
		file = decode_zstd(&file, Some(&frame_decoder)).unwrap();
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
	let _output = parse_blk(&file[1..], true, Some(Arc::new(shared_name_map))).unwrap();
}

/// Only run explicitly when required for testing, and do not run in CI because the test files it needs are not commited
#[test]
#[ignore]
fn test_all() {
	let start = Instant::now();
	let nm = fs::read("./samples/vromfs/aces.vromfs.bin_u/nm").unwrap();
	let dict = fs::read("./samples/vromfs/aces.vromfs.bin_u/ca35013aabca60792d5203b0137d0a8720d1dc151897eb856b12318891d08466.dict").unwrap();

	let frame_decoder = DecoderDictionary::copy(&dict);

	let dir: ReadDir = fs::read_dir("./samples/vromfs/aces.vromfs.bin_u").unwrap();
	let total = AtomicUsize::new(0);

	let mut pile = vec![];
	test_parse_dir(&mut pile, dir, &total);
	let shared_name_map = Some(Arc::new(NameMap::from_encoded_file(&nm).unwrap()));
	let arced_fd = Arc::new(frame_decoder);
	let out = pile
		.into_iter()
		.map(|file| parse_file(file.1, arced_fd.clone(), shared_name_map.clone()))
		.filter_map(|x| x)
		.collect::<Vec<_>>();

	let stop = start.elapsed();
	println!("Successfully parsed {} files! Thats all of them. The process took: {stop:?}", out.len());
}
