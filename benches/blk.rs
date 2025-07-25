use std::{fs, ops::Deref, path::Path, time::Duration};

use divan::{black_box, Bencher};
use wt_blk::{
	blk::{
		blk_string::blk_str,
		blk_structure::BlkField,
		blk_type::BlkType,
		make_strict_test,
		unpack_blk,
	},
	vromf::{File, VromfUnpacker},
};

fn main() {
	// Run registered benchmarks.
	divan::main();
}

static EXPECTED: &[u8] = include_bytes!("../samples/expected_merged.json");

#[divan::bench(
min_time = Duration::from_secs(3),
ignore
)]
fn streaming_merge(bencher: Bencher) {
	let unpacker =
		VromfUnpacker::from_file(&File::new("samples/aces.vromfs.bin").unwrap(), false).unwrap();
	let mut raw_blk = unpacker
		.unpack_one(
			Path::new("gamedata/units/tankmodels/germ_leopard_2a4.blk"),
			None,
			false,
		)
		.unwrap();
	let mut blk = unpack_blk(&mut raw_blk.buf_mut(), unpacker.dict(), unpacker.nm()).unwrap();

	bencher.bench_local(move || {
		black_box(&mut blk).merge_fields();
	});
}

#[divan::bench(
min_time = Duration::from_secs(3),
)]
fn general_unpack(bencher: Bencher) {
	let unpacker =
		VromfUnpacker::from_file(&File::new("samples/aces.vromfs.bin").unwrap(), false).unwrap();
	let mut raw_blk = unpacker
		.unpack_one(
			Path::new("gamedata/units/tankmodels/germ_leopard_2a4.blk"),
			None,
			false,
		)
		.unwrap();

	bencher
		.with_inputs(|| raw_blk.clone())
		.bench_local_values(move |mut f| {
			unpack_blk(
				black_box(&mut f.buf_mut()),
				black_box(unpacker.dict()),
				black_box(unpacker.nm()),
			)
			.unwrap();
		});
}
