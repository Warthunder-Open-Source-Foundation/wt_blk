use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use crate::blk::BlkOutputFormat;
use crate::blk::output_formatting_conf::FormattingConfiguration;
use crate::vromf::enums::VromfType;
use crate::vromf::unpacker::VromfUnpacker;

#[test]
fn grp_vromf() {
	let start = Instant::now();
	let p = PathBuf::from_str("./samples/grp_hdr.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file), VromfType::Grp).unwrap();
	let unpacked = out.unpack_all(Some(BlkOutputFormat::Json(FormattingConfiguration::GSZABI_REPO))).unwrap();
	assert_eq!(2322, unpacked.len())
}

#[test]
fn regular_vromf() {
	let start = Instant::now();
	let p = PathBuf::from_str("./samples/aces.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file), VromfType::Regular).unwrap();
	let unpacked = out.unpack_all(Some(BlkOutputFormat::Json(FormattingConfiguration::GSZABI_REPO))).unwrap();
	assert_eq!(15632, unpacked.len())
}

#[test]
fn no_nm_vromf() {
	let start = Instant::now();
	let p = PathBuf::from_str("./samples/atlases.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file), VromfType::Regular).unwrap();
	let unpacked = out.unpack_all(Some(BlkOutputFormat::Json(FormattingConfiguration::GSZABI_REPO))).unwrap();
	assert_eq!(8924, unpacked.len())
}