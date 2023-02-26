#[cfg(test)]
mod test {
	use std::fs;
	use crate::binary::blk_type::BlkType;
	use crate::binary::leb128::uleb128;
	use crate::binary::parser::parse_blk;

	#[test]
	fn fat_blk() {
		let file = fs::read("./samples/section_fat.blk").unwrap();
		let output = parse_blk(&file);
		println!("{:#?}", output);
	}
}