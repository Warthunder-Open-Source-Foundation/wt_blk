use crate::binary::blk_structure::BlkField;
use crate::binary::blk_type::BlkType;
use crate::binary::file::FileType;
use crate::binary::leb128::uleb128;
use crate::output_parsing::WTBlk;

pub fn parse_blk(file: &[u8], with_magic_byte: bool) -> (
	Vec<(usize, BlkField)>,
	Vec<(String, u8, u8, Option<u8>)>
) {
	let mut ptr = 0;

	if with_magic_byte {
		let file_type = FileType::from_byte(file[0]).unwrap();
		ptr += 1;
	}

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

	let params_data = &file[ptr..(ptr + params_data_size)];
	ptr += params_data_size;

	let params_info = &file[ptr..(ptr + params_count * 8)];
	ptr += params_info.len();

	let block_info = &file[ptr..];
	drop(ptr);

	let dbg_hex = |x: &[u8]| x.iter().map(|item| format!("{:X}", item)).collect::<Vec<String>>().join(" ");

	let mut results: Vec<(usize, BlkField)> = vec![];
	for chunk in params_info.chunks(8) {
		let name_id_raw = &chunk[0..3];
		let name_id = u32::from_le_bytes([
			name_id_raw[0],
			name_id_raw[1],
			name_id_raw[2],
			0
		]);
		let type_id = chunk[3];
		let data = &chunk[4..];
		let name = &names[name_id as usize];

		let parsed = BlkType::from_raw_param_info(type_id, data, params_data).unwrap();
		let field = BlkField::Value(name.to_owned(), parsed);
		results.push((name_id as usize, field));
	}

	let mut blocks = vec![];
	{
		let block_id_to_name = |id| {
			if id == 0 {
				"root".to_owned()
			} else {
				(&names)[(id - 1) as usize].clone()
			}
		};
		let mut stage = 0_u8;

		let mut name = 0_u8;
		let mut param_count = 0_u8;
		let mut blocks_count = 0_u8;
		let mut first_block_id = 0_u8;


		for (i, byte) in block_info.iter().enumerate() {
			/*println!("i: {i} stage: {} {} {} {} {}",
			stage, name, param_count, blocks_count, first_block_id);

			 */
			match stage {
				0 => {
					name = *byte;
					stage = 1;
				}
				1 => {
					param_count = *byte;
					stage = 2;
				}
				2 => {
					blocks_count = *byte;

					if blocks_count == 0 {
						blocks.push((block_id_to_name(name), param_count, blocks_count, None));
						name = 0;
						param_count = 0;
						blocks_count = 0;
						first_block_id = 0;
						stage = 0;
					} else {
						stage = 3;
					}
				}
				3 => {
					first_block_id = *byte;
					blocks.push((block_id_to_name(name), param_count, blocks_count, Some(first_block_id)));
					name = 0;
					param_count = 0;
					blocks_count = 0;
					first_block_id = 0;
					stage = 0;
				}
				_ => {
					unimplemented!();
				}
			}
		}
	}
	(results, blocks)
}