use std::fs;
use crate::blk::binary_deserialize::parser::parse_blk;
use crate::blk::make_strict_test;

/// Exports core function for unpacking BLK file
pub mod parser;


#[cfg(test)]
mod test {
	use std::fs;
	use crate::blk::binary_deserialize::parser::parse_blk;
	use crate::blk::make_strict_test;

	#[test]
	fn fat_blk() {
		let file = fs::read("./samples/temp.rez").unwrap();
		let output = parse_blk(&file[1..], false, None).unwrap();
		assert!(false)
	}
}
