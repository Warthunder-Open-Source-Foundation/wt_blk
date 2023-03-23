use std::borrow::Cow;
use std::ops::DerefMut;
use std::rc::Rc;
use std::time::Instant;
use crate::binary::blk_block_hierarchy::FlatBlock;
use crate::binary::blk_structure::BlkField;
use crate::binary::blk_type::{BlkString, BlkType};
use crate::binary::file::FileType;
use crate::binary::leb128::uleb128;
use crate::binary::nm_file::NameMap;

pub fn parse_blk(file: & [u8], with_magic_byte: bool, is_slim: bool, shared_name_map: Rc<NameMap>) -> BlkField {
	let mut ptr = 0;

	if with_magic_byte {
		let file_type = FileType::from_byte(file[0]).unwrap();
		ptr += 1;
	}

	let (offset, names_count) = uleb128(&file[ptr..]).unwrap();
	ptr += offset;


	let names = if is_slim { // TODO Figure out if names_count dictates the existence of a name map or if it may be 0 without requiring a name map
		shared_name_map.parsed.clone()
	} else {
		let (offset, names_data_size) = uleb128(&file[ptr..]).unwrap();
		ptr += offset;

		let names = NameMap::parse_name_section(&file[ptr..(ptr + names_data_size)]);
		ptr += names_data_size;
		if names_count != names.len() {
			panic!("Should be equal"); // TODO: Change to result when fn signature allows for it
		}
		Rc::new(names)
	};

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



	let mut results: Vec<(usize, BlkField)> = Vec::with_capacity(params_info.len() / 8);
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


		// TODO: Validate wether or not slim files store only strings in the name map
		let parsed = if is_slim && type_id == 0x01 {
			BlkType::from_raw_param_info(type_id, data, shared_name_map.binary.clone(),shared_name_map.parsed.clone()).unwrap()
		} else {
			BlkType::from_raw_param_info(type_id, data, Rc::new(params_data.to_owned()), names.clone()).unwrap()
		};

		let field = BlkField::Value(name.to_owned(), parsed);
		results.push((name_id as usize, field));
	}


	let mut blocks = vec![];
	{
		let block_id_to_name = |id| {
			if id == 0 {
				Rc::new("root".to_owned())
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
			// Name of the block
			// Amount of non-block fields
			// Amount of child-blocks
			// If it has child-blocks, starting index of said block

		}
	}

	// Create a flat hierarchy of all blocks including their non-block fields
	// This ensures all values are actually assigned
	// After this, the hierarchy will be assigned depth depending on the block-map
	let mut flat_map: Vec<FlatBlock> = Vec::with_capacity(blocks_count);
	let mut ptr = 0;
	for (name, field_count, blocks , offset) in blocks {
		let mut field = FlatBlock {
			name,
			fields: Vec::with_capacity(field_count),
			blocks,
			offset: offset.unwrap_or(0),
		};
		for i in (ptr)..(ptr + field_count) {
			field.fields.push(results[i].1.clone());
		}
		ptr += field_count;
		flat_map.push(field);
	}

	let out = BlkField::from_flat_blocks(flat_map);
	out
}