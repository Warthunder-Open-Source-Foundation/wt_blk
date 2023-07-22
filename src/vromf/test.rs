use std::{fs, path::PathBuf, str::FromStr, time::Instant};

use crate::{
	blk::{output_formatting_conf::FormattingConfiguration, BlkOutputFormat},
	vromf::unpacker::VromfUnpacker,
};

#[test]
fn grp_vromf() {
	let start = Instant::now();
	let p = PathBuf::from_str("./samples/grp_hdr.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file)).unwrap();
	let unpacked = out
		.unpack_all(Some(BlkOutputFormat::Json(
			FormattingConfiguration::GSZABI_REPO,
		)))
		.unwrap();
	assert_eq!(2322, unpacked.len())
}

#[test]
fn regular_vromf() {
	let start = Instant::now();
	let p = PathBuf::from_str("./samples/aces.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file)).unwrap();
	let unpacked = out
		.unpack_all(Some(BlkOutputFormat::Json(
			FormattingConfiguration::GSZABI_REPO,
		)))
		.unwrap();
	assert_eq!(15632, unpacked.len())
}

#[test]
fn no_nm_vromf() {
	let start = Instant::now();
	let p = PathBuf::from_str("./samples/atlases.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file)).unwrap();
	let unpacked = out
		.unpack_all(Some(BlkOutputFormat::Json(
			FormattingConfiguration::GSZABI_REPO,
		)))
		.unwrap();
	assert_eq!(8924, unpacked.len())
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