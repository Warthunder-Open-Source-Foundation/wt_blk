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
	let file = fs::read("./samples/grp_hdr.vromfs.bin").unwrap();
	// let out = decode_vromf(&file, VromfType::Grp).unwrap();
}

#[test]
fn regular_vromf() {
	let start = Instant::now();
	let p = PathBuf::from_str("./samples/aces.vromfs.bin").unwrap();
	let file = fs::read(&p).unwrap();
	let out = VromfUnpacker::from_file((p, file), VromfType::Regular).unwrap();
	println!("{:?}", start.elapsed());
	let unpacked = out.unpack_all(Some(BlkOutputFormat::Json(FormattingConfiguration::GSZABI_REPO))).unwrap();
}