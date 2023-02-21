

#[cfg(test)]
mod test {
	use crate::binary::file::FatBLk;
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
		
		let mut names = vec![];

		{
			let mut buff = vec![];
			for idx in 0..names_data_size {
				let char = file[ptr + idx];
				if char == 0 {
					names.push(String::from_utf8(buff.clone()).unwrap());
					buff.clear();
				} else {
					buff.push(char);
				}
			}
			ptr += names_data_size;
		}

		let (offset, blocks_count) = uleb128(&file[ptr..]).unwrap();
		ptr += offset;

		let (offset, params_count) = uleb128(&file[ptr..]).unwrap();
		ptr += offset;

		let (offset, params_data_size) = uleb128(&file[ptr..]).unwrap();
		ptr += offset;

		let blk = FatBLk {
			names_count,
			names_data_size,
			names,
			blocks_count,
			params_count,
			params_data_size,
		};

		let params_data = &file[ptr..(ptr + params_data_size)];
		ptr += params_data_size;



		println!("{:#?}", blk);
		println!("{:?}", &file[ptr..(ptr + params_data_size)].iter().map(|x|format!("{:x}",x)).collect::<Vec<String>>());
	}
}