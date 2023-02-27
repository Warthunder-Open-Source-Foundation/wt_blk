use std::fs;

use criterion::{black_box, Criterion, criterion_group, criterion_main};
use wt_blk::binary::parser::parse_blk;
use wt_blk::binary::zstd::decode_zstd;
use wt_blk::binary::zstd::eep;


pub fn blk_fat(c: &mut Criterion) {
	let f = fs::read("./samples/section_fat.blk").unwrap();
	c.bench_function("blk fat", |b| b.iter(|| parse_blk(black_box(&f), true)));
}

pub fn zstd(c: &mut Criterion) {
	let f = fs::read("./samples/section_fat_zst.blk").unwrap();
	c.bench_function("zstd", |b| b.iter(|| decode_zstd(black_box(&f))));
}

pub fn eep_bench(c: &mut Criterion) {
	c.bench_function("eep", |b| b.iter(|| eep()));
}


criterion_group!(benches, blk_fat, zstd, eep_bench);
criterion_main!(benches);
