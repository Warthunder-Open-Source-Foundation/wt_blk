













#[cfg(test)]
mod test {
	use std::fs;
	#[test]
	fn repack_fat_blk() {



		let _reference = fs::read("./samples/section_fat.blk").unwrap();
	}
}
