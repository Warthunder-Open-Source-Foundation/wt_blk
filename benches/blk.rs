use std::fs;
use std::ops::Deref;
use std::path::Path;
use std::time::Duration;
use divan::{black_box, Bencher};
use wt_blk::blk::{make_strict_test, unpack_blk};
use wt_blk::blk::blk_structure::BlkField;
use wt_blk::blk::blk_type::BlkType;
use wt_blk::blk::util::blk_str;
use wt_blk::vromf::{File, VromfUnpacker};

fn main() {
	// Run registered benchmarks.
	divan::main();
}

static EXPECTED: &[u8] = include_bytes!("../samples/expected_merged.json");

#[divan::bench(
min_time = Duration::from_secs(3),
)]
fn streaming_merge(bencher: Bencher) {
	let unpacker = VromfUnpacker::from_file(&File::new("samples/aces.vromfs.bin").unwrap(), false).unwrap();
	let mut raw_blk = unpacker.unpack_one(Path::new("gamedata/units/tankmodels/germ_leopard_2a4.blk"), None, false).unwrap();
	let mut blk = unpack_blk(&mut raw_blk.buf_mut(), unpacker.dict(), unpacker.nm()).unwrap();

	bencher.bench_local(move || {
		black_box(&mut blk).merge_fields();
	});

}
