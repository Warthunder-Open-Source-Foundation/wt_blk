use std::fmt;
use std::fmt::Write;
use color_eyre::eyre::ensure;
use color_eyre::Report;

#[inline(always)]
pub(crate)  fn bytes_to_offset(input: &[u8]) -> Result<usize, Report> {
	ensure!(input.len() == 4, "Invalid input-length, found {}", input.len());
	Ok(u32::from_le_bytes([input[0], input[1], input[2], input[3]]) as usize)
}

#[inline(always)]
pub(crate)  fn bytes_to_float(input: &[u8]) -> Result<f32, Report> {
	ensure!(input.len() == 4, "Invalid input-length, found {}", input.len());
	Ok(f32::from_le_bytes([input[0], input[1], input[2], input[3]]))
}

#[inline(always)]
pub(crate)  fn bytes_to_int(input: &[u8]) -> Result<u32, Report> {
	ensure!(input.len() == 4, "Invalid input-length, found {}", input.len());
	Ok(u32::from_le_bytes([input[0], input[1], input[2], input[3]]))
}

#[inline(always)]
pub(crate)  fn bytes_to_long(input: &[u8]) -> Result<u64, Report> {
	ensure!(input.len() == 8, "Invalid input-length, found {}", input.len());
	Ok(u64::from_le_bytes([
		input[0], input[1], input[2], input[3], input[4], input[5], input[6], input[7],
	]))
}

#[inline(always)]
//													('\t', 1) or (' ', 4) are typical
pub(crate) fn indent(f: &mut String, depth: usize, (with, amount): (char, usize)) -> Result<(), fmt::Error> {
	for _ in 0..(depth * amount) {
		f.write_char(with)?;
	}

	Ok(())
}

#[macro_export]
macro_rules! make_thing {
    // entry point
    ($writer:expr, $($things:expr),+) => {
        writeln!($writer, "wcall start")?;
        make_thing!(@recourse, $writer, $($things),+)
    };
    // recursive exit
    (@recourse, $writer:expr, $thing:expr) => {
        make_thing!(@item, $writer, $thing);
        make_thing!(@end, $writer);
    };
    // recursing
    (@recourse, $writer:expr, $thing:expr, $($other:expr),+) => {
        make_thing!(@item, $writer, $thing);
        make_thing!(@recourse, $writer, $($other),*);
    };
    // item write
    (@item, $writer:expr, $thing:expr) => {
      writeln!($writer, "thing = {}", $thing)?;
    };
    // terminal
    (@end, $writer:expr) => {
        writeln!($writer, "wcall end")?;
    };
}

#[cfg(test)]
mod test {
	use std::fmt;
	use crate::blk::util::indent;
	use std::fmt::Write;

	#[test]
	fn interlace_writer_string() -> Result<(), fmt::Error>{
		let mut f = String::new();
		make_thing!(&mut f, "yeet", "yeet 2 electric bogaloo");
		Ok(())
	}
}