#[allow(unused)]
macro_rules! time {
	($e:expr) => {{
		let start = std::time::Instant::now();
		let output = $e;
		println!("{}:{} {:?}", file! {}, line! {}, start.elapsed());
		drop(start);
		output
	}};
}

use itertools::Itertools;
#[allow(unused)]
pub(crate) use time;

#[allow(unused)]
pub(crate) fn format_hex(hex: &[u8]) -> Vec<String> {
	hex.iter().map(|x| format!("0x{x:X}")).collect::<Vec<_>>()
}

#[allow(unused)]
pub(crate) fn join_hex(hex: &[u8]) -> String {
	hex.iter().map(|x| format!("{x:x}")).join("")
}

#[allow(unused)]
pub(crate) fn debug_hex(hex: &[u8]) {
	eprintln!("{:?}", format_hex(hex));
}