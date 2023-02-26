use crate::binary::blk_structure::BlkField;
use crate::binary::blk_type::BlkType;
use crate::binary::file::FileType;
use crate::binary::leb128::uleb128;
use crate::output_parsing::WTBlk;

pub fn parse_blk(file: &[u8], with_magic_byte: bool) -> (
	Vec<(usize, BlkField)>,
	Vec<(String, usize, usize, Option<usize>)>
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

		let mut ptr = 0;
		for _ in 0..blocks_count {
			let (offset, name_id) = uleb128(&block_info[ptr..]).unwrap();
			ptr += offset;

			let (offset, param_count) = uleb128(&block_info[ptr..]).unwrap();
			ptr += offset;

			let (offset, blocks_count) = uleb128(&block_info[ptr..]).unwrap();
			ptr += offset;

			let first_block_id = if blocks_count > 0 {
				let (offset, first_block_id) = uleb128(&block_info[ptr..]).unwrap();
				ptr += offset;
				Some(first_block_id)
			} else {
				None
			};

			blocks.push((block_id_to_name(name_id), param_count, blocks_count, first_block_id));

		}
	}
	(results, blocks)
}