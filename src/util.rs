
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

#[allow(unused)]
pub(crate) use time;

#[allow(unused)]
pub(crate) fn debug_hex(hex: &[u8]) {
	eprintln!(
		"{:?}",
		hex.iter().map(|x| format!("0x{x:X}")).collect::<Vec<_>>()
	);
}
