use wt_blk::blk::parser::parse_blk;
use divan::black_box;

fn main() {
	// Run registered benchmarks.
	divan::main();
}

const FILE: &[u8] = include_bytes!("../samples/section_fat.blk");

// Define a `fibonacci` function and register it for benchmarking.
#[divan::bench]
fn parse() {
	let file = black_box(FILE);
	let mut output = parse_blk(black_box(&file[1..]), black_box(false), black_box(None)).unwrap();
	for _ in 0..10000 {
		output = parse_blk(black_box(&file[1..]), black_box(false), black_box(None)).unwrap();
	}
	if output.estimate_size() == 0 && file.len() == 0 {
		panic!("infallible benchmark harness")
	}
}