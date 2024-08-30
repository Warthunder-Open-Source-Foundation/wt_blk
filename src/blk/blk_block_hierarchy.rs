use std::ops::Range;
use iex::iex;
use crate::blk::{blk_structure::BlkField, blk_type::BlkString};
use crate::blk::error::BlkError;

#[derive(Debug, Clone)]
pub struct FlatBlock {
	pub name:   BlkString,
	pub fields: Vec<BlkField>,
	pub blocks: usize,
	pub offset: usize,
}

impl FlatBlock {
	fn location_range(&self) -> Range<usize> {
		self.offset..(self.offset + self.blocks)
	}
}

impl BlkField {
	#[iex]
	pub fn from_flat_blocks(
		flat_blocks: &mut Vec<Option<FlatBlock>>,
	) -> Result<Self, BlkError> {
		let cloned = flat_blocks[0].take().ok_or("Initial element missing for block map")?;
		let ret = Self::from_flat_blocks_with_parent(flat_blocks, cloned)?;

		#[cfg(debug_assertions)]
		if flat_blocks.into_iter().all(|e| e.is_none()) == false {
			return Err("Unclaimed elements remain in block hierarchy");
		}

		Ok(ret)
	}

	#[iex]
	fn from_flat_blocks_with_parent(
		flat_blocks: &mut Vec<Option<FlatBlock>>,
		parent: FlatBlock,
	) -> Result<Self, BlkError> {
		let range = parent.location_range();
		let mut block = BlkField::Struct(parent.name, parent.fields);

		let block_range = flat_blocks[range]
			.iter_mut()
			.map(|e| e.take())
			.collect::<Option<Vec<FlatBlock>>>()
			.ok_or("taken element missing")?;

		for flat_block in block_range {
			block
				.insert_field(Self::from_flat_blocks_with_parent(flat_blocks, flat_block)?)
				.ok_or("inserting into non-struct")?;
		}

		Ok(block)
	}
}
