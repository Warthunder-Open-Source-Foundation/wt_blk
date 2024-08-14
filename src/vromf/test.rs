use std::{fs, path::PathBuf, str::FromStr};

use wt_version::Version;

use crate::vromf::{
	binary_container::decode_bin_vromf,
	inner_container::decode_inner_vromf,
	unpacker::{BlkOutputFormat, VromfUnpacker, ZipFormat},
};

#[test]
fn grp_vromf() {
	let p = PathBuf::from_str("./samples/grp_hdr.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file), true).unwrap();
	let unpacked = out.unpack_all(Some(BlkOutputFormat::Json), true).unwrap();
	assert_eq!(2322, unpacked.len())
}

#[test]
fn write_to_zip() {
	let p = PathBuf::from_str("./samples/aces.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file), true).unwrap();
	let unpacked = out
		.unpack_all_to_zip(ZipFormat::Compressed(1), Some(BlkOutputFormat::Json), true)
		.unwrap();
	assert_eq!(62873033, unpacked.len())
}

#[test]
fn regular_vromf() {
	let p = PathBuf::from_str("./samples/aces.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file), true).unwrap();
	let unpacked = out.unpack_all(None, true).unwrap();
	assert_eq!(15632, unpacked.len())
}

// Smoke-test
#[test]
fn regional() {
	let p = PathBuf::from_str("./samples/regional.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file), true).unwrap();
	let _unpacked = out
		.unpack_one(
			&PathBuf::from_str("dldata/downloadable_decals.blk").unwrap(),
			Some(BlkOutputFormat::BlkText),
			true,
		)
		.unwrap();
}

#[test]
fn no_nm_vromf() {
	let p = PathBuf::from_str("./samples/atlases.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file), true).unwrap();
	let unpacked = out.unpack_all(Some(BlkOutputFormat::Json), true).unwrap();
	assert_eq!(8924, unpacked.len())
}

#[test]
fn decode_simple() {
	let f = fs::read("./samples/checked_simple_uncompressed_checked.vromfs.bin").unwrap();
	let (decoded, _) = decode_bin_vromf(&f, true).unwrap();
	let _ = decode_inner_vromf(&decoded).unwrap();
}

#[test]
fn version() {
	let p = PathBuf::from_str("./samples/aces.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file), true).unwrap();
	assert_eq!(
		vec![
			Version::from_str("2.25.1.39").unwrap(),
			Version::from_str("2.25.1.39").unwrap()
		],
		out.query_versions().unwrap()
	);
}

// New format
#[test]
fn new_format() {
	let p = PathBuf::from_str("./samples/2_30_vromfs/aces.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file), true).unwrap();
	let _unpacked = out.unpack_all(None, false).unwrap();
	println!("{}", _unpacked.len());
}

// Used for bugfixing, re-enable when this file acts up again
// #[test]
// fn new_char() {
// 	load_eyre();
// 	let p = PathBuf::from_str("./samples/char.vromfs1.bin").unwrap();
// 	let file = fs::read(&p).unwrap();
// 	let out = VromfUnpacker::from_file((p, file)).unwrap();
// 	let unpacked = out
// 		.unpack_all(Some(BlkOutputFormat::Json(
// 			FormattingConfiguration::GSZABI_REPO,
// 		)))
// 		.unwrap();
// }
