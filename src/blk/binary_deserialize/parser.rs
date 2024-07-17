use std::{borrow::Cow, sync::Arc};

use tracing::error;

use crate::blk::{
	blk_block_hierarchy::FlatBlock,
	blk_structure::BlkField,
	blk_type::{blk_type_id::STRING, BlkString, BlkType},
	error::{
		ParseError,
		ParseError::{BadBlkValue, ResidualBlockBuffer},
	},
	leb128::uleb128,
	nm_file::NameMap,
	util::blk_str,
};

/// Lowest-level function which unpacks BLK to [`crate::blk::blk_structure::BlkField`]
pub fn parse_blk(
	file: &[u8],
	is_slim: bool,
	shared_name_map: Option<Arc<NameMap>>,
) -> Result<BlkField, ParseError> {
	let mut ptr = 0;

	// Globally increments ptr and returns next uleb integer from file
	let next_uleb = |ptr: &mut usize| {
		// Using ? inside of closures is not supported yet, so we need to use this match
		match uleb128(&file[*ptr..]) {
			Ok((offset, int)) => {
				*ptr += offset;
				Ok(int)
			},
			Err(e) => Err(e),
		}
	};

	// Returns slice offset from file, incrementing the ptr by offset
	let idx_file_offset = |ptr: &mut usize, offset: usize| {
		let res = file
			.get(*ptr..(*ptr + offset))
			.ok_or(ParseError::DataRegionBoundsExceeded(*ptr..(*ptr + offset)));
		*ptr += offset;
		res
	};

	let names_count = next_uleb(&mut ptr)?;

	let names = if is_slim {
		// TODO Figure out if names_count dictates the existence of a name map or if it may be 0 without requiring a name map
		Cow::Borrowed(
			shared_name_map
				.as_deref()
				.ok_or(ParseError::SlimBlkWithoutNm)?
				.parsed
				.as_ref(),
		)
	} else {
		let names_data_size = next_uleb(&mut ptr)?;

		let names = NameMap::parse_name_section(idx_file_offset(&mut ptr, names_data_size)?);
		if names_count != names.len() {
			error!("Name count mismatch, expected {names_count}, but found a len of {}. This might mean something is wrong.", names.len());
		}
		Cow::Owned(names)
	};

	let blocks_count = next_uleb(&mut ptr)?;

	let params_count = next_uleb(&mut ptr)?;

	let params_data_size = next_uleb(&mut ptr)?;

	let params_data = idx_file_offset(&mut ptr, params_data_size)?;

	let params_info = idx_file_offset(&mut ptr, params_count * 8)?;

	let block_info = &file.get(ptr..).ok_or(ResidualBlockBuffer)?;

	let _ptr = (); // Shadowing ptr causes it to become unusable, especially on accident

	let mut results: Vec<(usize, Option<BlkField>)> = Vec::with_capacity(params_info.len() / 8);

	let chunks = params_info.array_chunks::<8>();
	if chunks.remainder().len() != 0 {
		error!("Params info chunks did not align to 8 bytes")
	} // TODO: Decide whether or not this constitutes a hard crash
	for chunk in chunks {
		let name_id_raw = &chunk[0..3];
		let name_id = u32::from_le_bytes([name_id_raw[0], name_id_raw[1], name_id_raw[2], 0]);
		let type_id = chunk[3];
		let data = &chunk[4..];
		let name = names
			.get(name_id as usize)
			.ok_or(ParseError::Custom(format!(
				"Name index {} out of bounds for name map of length {}",
				name_id,
				names.len()
			)))?
			.clone();

		let parsed = if is_slim && type_id == STRING {
			BlkType::from_raw_param_info(
				type_id,
				data,
				shared_name_map
					.as_deref()
					.ok_or(ParseError::SlimBlkWithoutNm)?
					.binary
					.as_slice(),
				shared_name_map
					.as_deref()
					.ok_or(ParseError::SlimBlkWithoutNm)?
					.parsed
					.as_slice(),
			)
			.ok_or(BadBlkValue)?
		} else {
			BlkType::from_raw_param_info(type_id, data, params_data, names.as_ref())
				.ok_or(BadBlkValue)?
		};

		let field = BlkField::Value(name, parsed);
		results.push((name_id as usize, Some(field)));
	}

	let mut block_ptr = 0;
	let block_id_to_name = |id| {
		if id == 0 {
			blk_str("root")
		} else {
			(names)[(id - 1) as usize].clone()
		}
	};
	let blocks = {
		(0..blocks_count).into_iter().map(|_| {
			let (offset, name_id) = uleb128(&block_info[block_ptr..]).unwrap();
			block_ptr += offset;

			let (offset, param_count) = uleb128(&block_info[block_ptr..]).unwrap();
			block_ptr += offset;

			let (offset, blocks_count) = uleb128(&block_info[block_ptr..]).unwrap();
			block_ptr += offset;

			let first_block_id = if blocks_count > 0 {
				let (offset, first_block_id) = uleb128(&block_info[block_ptr..]).unwrap();
				block_ptr += offset;
				Some(first_block_id)
			} else {
				None
			};
			// Name of the block
			// Amount of non-block fields
			// Amount of child-blocks
			// If it has child-blocks, starting index of said block
			(
				block_id_to_name(name_id),
				param_count,
				blocks_count,
				first_block_id,
			)
		})
	};

	// Create a flat hierarchy of all blocks including their non-block fields
	// This ensures all values are actually assigned
	// After this, the hierarchy will be assigned depth depending on the block-map
	let mut flat_map: Vec<Option<FlatBlock>> = Vec::with_capacity(blocks_count);
	let mut ptr = 0;
	for (name, field_count, blocks, offset) in blocks {
		let mut field = FlatBlock {
			name,
			fields: Vec::with_capacity(field_count),
			blocks,
			offset: offset.unwrap_or(0),
		};
		for i in (ptr)..(ptr + field_count) {
			field.fields.push(
				results[i]
					.1
					.take()
					.expect("Infallible, already taken value"),
			);
		}
		ptr += field_count;
		flat_map.push(Some(field));
	}
	#[cfg(debug_assertions)]
	assert!(
		results.into_iter().all(|e| e.1.is_none()),
		"unused values in results"
	);

	let out = BlkField::from_flat_blocks(&mut flat_map)
		.map_err(|e| ParseError::BlkBlockBuilderError(e))?;
	Ok(out)
}
