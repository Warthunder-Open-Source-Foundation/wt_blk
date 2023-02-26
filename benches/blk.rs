use std::fs;

use criterion::{black_box, Criterion, criterion_group, criterion_main};
use wt_blk::binary::parser::parse_blk;

pub fn blk_fat(c: &mut Criterion) {
	let f = fs::read("./samples/section_fat.blk").unwrap();
	c.bench_function("blk fat", |b| b.iter(|| parse_blk(black_box(&f))));
}

criterion_group!(benches, blk_fat);
criterion_main!(benches);
