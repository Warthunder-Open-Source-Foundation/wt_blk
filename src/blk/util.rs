use std::any::type_name;
use std::fmt;
use std::fmt::Write;
use std::mem::{align_of, size_of};
use color_eyre::eyre::ensure;
use color_eyre::Report;
use crate::blk::error::BlkTypeError;

#[inline(always)]
pub(crate)  fn bytes_to_offset(input: &[u8]) -> Result<usize, BlkTypeError> {
	assure_len::<u32>(input)?;
	Ok(u32::from_le_bytes([input[0], input[1], input[2], input[3]]) as usize)
}

#[inline(always)]
pub(crate)  fn bytes_to_float(input: &[u8]) -> Result<f32, BlkTypeError> {
	assure_len::<f32>(input)?;
	Ok(f32::from_le_bytes([input[0], input[1], input[2], input[3]]))
}

#[inline(always)]
pub(crate)  fn bytes_to_int(input: &[u8]) -> Result<u32, BlkTypeError> {
	assure_len::<u32>(input)?;
	Ok(u32::from_le_bytes([input[0], input[1], input[2], input[3]]))
}

#[inline(always)]
pub(crate)  fn bytes_to_long(input: &[u8]) -> Result<u64, BlkTypeError> {
	assure_len::<u64>(input)?;
	Ok(u64::from_le_bytes([
		input[0], input[1], input[2], input[3], input[4], input[5], input[6], input[7],
	]))
}

fn assure_len<T>(buf: &[u8]) -> Result<(), BlkTypeError> {
	if buf.len() != align_of::<T>() {
		Err(BlkTypeError::NumberSizeMissmatch { found: buf.len(), expected: type_name::<T>() })
	} else {
		Ok(())
	}
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