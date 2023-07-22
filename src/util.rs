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

use std::env;
#[allow(unused)]
pub(crate) use time;

#[allow(unused)]
pub(crate) fn format_hex(hex: &[u8]) -> Vec<String> {
	hex.iter().map(|x| format!("0x{x:X}")).collect::<Vec<_>>()
}

#[allow(unused)]
pub(crate) fn debug_hex(hex: &[u8]) {
	eprintln!(
		"{:?}",
		format_hex(hex)
	);
}

#[allow(unused)]
#[cfg(test)]
pub(crate) fn load_eyre() {
	env::set_var("RUST_BACKTRACE", "full");
	color_eyre::install().unwrap();

}
