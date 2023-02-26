#[cfg(test)]
mod test {
	use std::time::Instant;
	use crate::binary::blk_type::BlkType;
	use crate::binary::leb128::uleb128;
	use crate::binary::parser::parse_blk;

	#[test]
	fn fat_blk() {
		let file = include_bytes!("../../samples/section_fat.blk");
		let start = Instant::now();
		let output = parse_blk(file);
		println!("{:?}", start.elapsed());
		println!("{:?}", output);
	}
}