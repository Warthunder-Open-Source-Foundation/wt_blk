

#[cfg(test)]
mod test {
	use crate::binary::leb128::uleb128;

	#[test]
	fn fat_blk() {
		let file = include_bytes!("../../samples/section_fat.blk");
		let mut ptr = 0;

		let file_type = file[0];
		ptr += 1;

		let (offset, names_count) = uleb128(&file[ptr..]).unwrap();
		ptr += offset;

		let (offset, names_data_size) = uleb128(&file[ptr..]).unwrap();
		ptr += offset;
	}
}