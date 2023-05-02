use std::fs;
use crate::vromf::decode_vromf;
use crate::vromf::enums::FileMode;

#[test]
fn grp_vromf() {
	let file = fs::read("./samples/grp_hdr.vromfs.bin").unwrap();
	let out = decode_vromf(&file, FileMode::Grp).unwrap();
}