#[cfg(test)]
mod test {
	use std::fs;
	use std::mem::size_of;
	use crate::binary::blk_type::BlkType;
	use crate::binary::leb128::uleb128;
	use crate::binary::nm_file::{decode_nm_file, parse_name_section};
	use crate::binary::parser::parse_blk;

	#[test]
	fn fat_blk() {
		let file = fs::read("./samples/section_fat.blk").unwrap();
		let output = parse_blk(&file, true, false,None);
		// println!("{:#?}", output);
	}

	#[test]
	fn fat_blk_router_probe() {
		let file = fs::read("./samples/route_prober.blk").unwrap();
		let output = parse_blk(&file, false, false,None);
		// println!("{:?}", output);
	}

	#[test]
	fn slim_blk() {
		let file = fs::read("./samples/section_slim.blk").unwrap();
		let nm = fs::read("./samples/names").unwrap();
		let output = parse_blk(&file, true, true,Some(&nm));
		// println!("{:?}", output);
	}
}