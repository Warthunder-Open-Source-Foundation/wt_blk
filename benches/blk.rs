use std::fs;
use std::rc::Rc;

use criterion::{black_box, Criterion, criterion_group, criterion_main};
use ruzstd::FrameDecoder;
use wt_blk::blk::leb128::uleb128;
use wt_blk::blk::nm_file::{decode_nm_file, parse_name_section};
use wt_blk::blk::parser::parse_blk;
use wt_blk::blk::zstd::decode_zstd;
use wt_blk::blk::zstd::eep;


pub fn blk_fat(c: &mut Criterion) {
	let f = fs::read("./samples/section_fat.blk").unwrap();
	c.bench_function("blk fat", |b| b.iter(|| parse_blk(black_box(&f), true, false, None, Rc::new(vec![]))));
}

pub fn zstd(c: &mut Criterion) {
	let f = fs::read("./samples/section_fat_zst.blk").unwrap();
	c.bench_function("zstd", |b| b.iter(|| decode_zstd(black_box(&f), Rc::new(FrameDecoder::new()))));
}

pub fn nm_section(c: &mut Criterion) {
	let nm = fs::read("../wt_blk/samples/rendist/nm").unwrap();
	let nm = decode_nm_file(&nm).unwrap();

	let mut nm_ptr = 0;

	let (offset, names_count) = uleb128(&nm[nm_ptr..]).unwrap();
	nm_ptr += offset;

	let (offset, names_data_size) = uleb128(&nm[nm_ptr..]).unwrap();
	nm_ptr += offset;

	c.bench_function("nm_section", |b| b.iter(|| parse_name_section(black_box(&nm[nm_ptr..(nm_ptr + names_data_size)]))));
}

pub fn eep_bench(c: &mut Criterion) {
	c.bench_function("eep", |b| b.iter(|| eep()));
}


criterion_group!(benches, nm_section);
criterion_main!(benches);
