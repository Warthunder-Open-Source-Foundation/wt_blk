#[cfg(test)]
mod test {
	use std::fs;
	use std::mem::size_of;
	use crate::binary::blk_type::BlkType;
	use crate::binary::leb128::uleb128;
	use crate::binary::parser::parse_blk;

	#[test]
	fn fat_blk() {
		let file = fs::read("./samples/section_fat.blk").unwrap();
		let output = parse_blk(&file, true, None);
		println!("{:#?}", output);
	}

	#[test]
	fn fat_blk_router_probe() {
		let file = fs::read("./samples/route_prober.blk").unwrap();
		let output = parse_blk(&file, false, None);
		println!("{:?}", output);
	}

	#[test]
	fn size_validator() {
		println!("{}", size_of::<BlkType>());
	}
}