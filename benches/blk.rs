use std::fs;
use divan::black_box;
use wt_blk::blk::{make_strict_test};
use wt_blk::blk::blk_structure::BlkField;
use wt_blk::blk::blk_type::BlkType;
use wt_blk::blk::util::blk_str;

fn main() {
	// Run registered benchmarks.
	divan::main();
}

static EXPECTED: &[u8] = include_bytes!("../samples/expected_merged.json");

#[divan::bench]
fn streaming_merge() {
	let mut blk = make_strict_test();
	blk.insert_field(BlkField::Value(blk_str("int"), BlkType::Int(420)))
		.unwrap();
	blk.merge_fields();
	let mut buf = vec![];
	blk.as_serde_json_streaming(&mut buf).unwrap();
	assert_eq!(
		buf,
		EXPECTED,
	);
}
