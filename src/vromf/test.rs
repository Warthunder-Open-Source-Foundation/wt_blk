use std::fs;
use crate::vromf::binary_container::FileMode;
use crate::vromf::decode_vromf;

#[test]
fn grp_vromf() {
	let file = fs::read("./samples/grp_hdr.vromfs.bin").unwrap();
	let out = decode_vromf(&file, FileMode::Grp).unwrap();
}