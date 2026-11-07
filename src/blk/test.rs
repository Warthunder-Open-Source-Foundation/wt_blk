use std::{fs, sync::Arc, time::Instant};

use zstd::dict::DecoderDictionary;

use crate::blk::{
	binary_deserialize::parser::{parse_blk, parse_blk_dedup},
	blk_string::blk_str,
	blk_structure::BlkField,
	blk_type::BlkType,
	file::FileType,
	leb128::uleb128_offset,
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

fn write_uleb(out: &mut Vec<u8>, mut v: usize) {
	loop {
		let byte = (v & 0x7F) as u8;
		v >>= 7;
		if v == 0 {
			out.push(byte);
			return;
		}
		out.push(byte | 0x80);
	}
}

/// Rewrites a FAT body into the FAT_DEDUP layout by adding explicit parameter offsets to every block
fn fat_to_dedup(fat: &[u8]) -> Vec<u8> {
	let mut ptr = 0;
	let names_count = uleb128_offset(&fat[ptr..], &mut ptr).unwrap();
	if names_count > 0 {
		let names_size = uleb128_offset(&fat[ptr..], &mut ptr).unwrap();
		ptr += names_size;
	}
	let blocks_count = uleb128_offset(&fat[ptr..], &mut ptr).unwrap();
	let params_count = uleb128_offset(&fat[ptr..], &mut ptr).unwrap();
	let params_data_size = uleb128_offset(&fat[ptr..], &mut ptr).unwrap();
	ptr += params_data_size + params_count * 8;

	let mut out = fat[..ptr].to_vec();
	let mut next_param = 0;
	for _ in 0..blocks_count {
		let name_id = uleb128_offset(&fat[ptr..], &mut ptr).unwrap();
		let param_count = uleb128_offset(&fat[ptr..], &mut ptr).unwrap();
		let child_count = uleb128_offset(&fat[ptr..], &mut ptr).unwrap();
		write_uleb(&mut out, name_id);
		write_uleb(&mut out, param_count);
		write_uleb(&mut out, child_count);
		if child_count > 0 {
			let first_block = uleb128_offset(&fat[ptr..], &mut ptr).unwrap();
			write_uleb(&mut out, first_block);
		}
		if param_count > 0 {
			write_uleb(&mut out, next_param);
			next_param += param_count;
		}
	}
	out
}

#[test]
fn fat_dedup_matches_fat() {
	let file = fs::read("./samples/section_fat.blk").unwrap();
	let dedup = fat_to_dedup(&file[1..]);
	assert_eq!(parse_blk_dedup(&dedup).unwrap(), make_strict_test());

	let mut with_header = vec![FileType::FAT_DEDUP as u8];
	with_header.extend_from_slice(&dedup);
	assert_eq!(
		unpack_blk(&mut with_header, None, None).unwrap(),
		make_strict_test()
	);
}

#[test]
fn fat_dedup_shared_params() {
	// root { a { x:i=7 } b { x:i=7 } }, where both blocks point at the same single parameter
	let mut file = vec![];
	let names = b"x\0a\0b\0";
	write_uleb(&mut file, 3);
	write_uleb(&mut file, names.len());
	file.extend_from_slice(names);
	write_uleb(&mut file, 3); // blocks
	write_uleb(&mut file, 1); // params
	write_uleb(&mut file, 0); // params data size
	file.extend_from_slice(&[0, 0, 0, 0x02]); // name 0 ("x"), type INT
	file.extend_from_slice(&7_i32.to_le_bytes());
	file.extend_from_slice(&[0, 0, 2, 1]); // root: no params, 2 children starting at block 1
	file.extend_from_slice(&[2, 1, 0, 0]); // "a": 1 param at offset 0
	file.extend_from_slice(&[3, 1, 0, 0]); // "b": 1 param at offset 0

	let child = |name: &str| {
		BlkField::Struct(
			blk_str(name),
			vec![BlkField::Value(blk_str("x"), BlkType::Int(7))],
		)
	};
	let expected = BlkField::Struct(blk_str("root"), vec![child("a"), child("b")]);
	assert_eq!(parse_blk_dedup(&file).unwrap(), expected);
}

#[test]
fn fat_dedup_rejects_out_of_bounds_offset() {
	let file = fs::read("./samples/section_fat.blk").unwrap();
	let mut dedup = fat_to_dedup(&file[1..]);
	// The last byte is the parameter offset of the last block, point it past the table
	*dedup.last_mut().unwrap() = 0x7F;
	let err = parse_blk_dedup(&dedup).unwrap_err().to_string();
	assert!(err.contains("out of bounds"), "{err}");
}

#[test]
fn fat_dedup_missing_parameter_offset_is_an_error() {
	let mut file = vec![1, 2, b'x', 0, 1, 1, 0];
	file.extend_from_slice(&[0, 0, 0, 2]);
	file.extend_from_slice(&7_i32.to_le_bytes());
	file.extend_from_slice(&[0, 1, 0, 0]);
	assert!(parse_blk_dedup(&file).is_ok());
	file.pop();
	assert!(parse_blk_dedup(&file).is_err());
}

#[test]
fn dedup_parameter_offset_cannot_wrap() {
	let mut file = vec![1, 2, b'x', 0, 1, 1, 0];
	file.extend_from_slice(&[0, 0, 0, 2]);
	file.extend_from_slice(&7_i32.to_le_bytes());
	file.extend_from_slice(&[0, 1, 0]);
	file.extend_from_slice(&[0xFF; 9]);
	file.push(1);
	assert!(parse_blk_dedup(&file).is_err());
}

#[test]
fn dedup_parameter_slice_uses_its_explicit_start() {
	let mut file = vec![1, 2, b'x', 0, 1, 2, 0];
	for value in [7_i32, 19] {
		file.extend_from_slice(&[0, 0, 0, 2]);
		file.extend_from_slice(&value.to_le_bytes());
	}
	file.extend_from_slice(&[0, 1, 0, 1]);
	let parsed = parse_blk_dedup(&file).unwrap();
	assert_eq!(
		parsed
			.pointer("x")
			.unwrap()
			.value()
			.unwrap()
			.to_string_default(),
		"i = 19"
	);
}
