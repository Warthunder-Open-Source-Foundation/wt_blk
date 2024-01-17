use wt_blk::blk::parser::parse_blk;
use wt_blk::blk::make_strict_test;
use divan::black_box;

fn main() {
	// Run registered benchmarks.
	divan::main();
}

const FILE: &[u8] = include_bytes!("../samples/section_fat.blk");

#[divan::bench]
fn to_string_bench() {
	let mut sample = make_strict_test();
	let mut out = String::new();
	for _ in 0..100_000 {
		out = serde_json::to_string_pretty(black_box(&sample.as_serde_obj(true))).unwrap();
	}
	if out.len() == 0 {
		panic!("infallible benchmark harness");
	}
}

#[divan::bench]
fn streaming_bench() {
	let sample = make_strict_test();
	let mut out = vec![];
	for _ in 0..100_000 {
		sample.as_serde_json_streaming(&mut out, false).unwrap();
	}
	if out.len() == 0 {
		panic!("infallible benchmark harness");
	}
}