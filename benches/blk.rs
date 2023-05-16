use std::path::PathBuf;
use std::str::FromStr;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wt_blk::blk::BlkOutputFormat;
use wt_blk::blk::output_formatting_conf::FormattingConfiguration;
use wt_blk::vromf::unpacker::VromfUnpacker;


fn blk_ref_json(c: &mut Criterion) {
	let vromf = include_bytes!("../samples/aces.vromfs.bin");
	let serialized = VromfUnpacker::from_file((PathBuf::from_str("../samples/aces.vromfs.bin").unwrap(), vromf.to_vec())).unwrap();
	c.bench_function("blk as_ref_json all", |b|{
		b.iter(
			|| {

			}
		);
	});
}


criterion_group!(benches, blk_ref_json);
criterion_main!(benches);
