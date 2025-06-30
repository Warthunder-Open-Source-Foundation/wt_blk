use std::time::{Duration, Instant};
use wt_blk::blk::binary_deserialize::parser::parse_blk;

static INPUT: &[u8] = include_bytes!("../../samples/section_fat.blk");

fn main() {
	let mut sum = 0;
	let dur = Instant::now();
	let timeout = 2.0;
	let mut iters = 0;
	
	for count in 0.. {
		let output = parse_blk(&INPUT[1..], false, None).unwrap();
		sum += output.get_name().len();
		if dur.elapsed().as_secs_f64() > timeout {
			iters = count;
			break;
		}
	};
	println!("{}", sum);
	let per = Duration::from_secs_f64(timeout) / iters;
	println!("completed {iters} iterations in {timeout} seconds, spending {per:?} per iteration. That makes {:.2} iterations per second", 1.0 / per.as_secs_f64());
}
