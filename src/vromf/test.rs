use std::fs;
use crate::vromf::enums::VromfType;

#[test]
fn grp_vromf() {
	let file = fs::read("./samples/grp_hdr.vromfs.bin").unwrap();
	// let out = decode_vromf(&file, VromfType::Grp).unwrap();
}