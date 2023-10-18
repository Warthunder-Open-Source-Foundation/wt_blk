use std::ops::Range;

use crate::blk::{blk_structure::BlkField, blk_type::BlkString};
use crate::blk::blk_block_hierarchy::BlkBlockBuilderError::{InsertingIntoNonStruct, TakenElementMissing, UnclaimedElements};

#[derive(Debug, thiserror::Error)]
pub enum BlkBlockBuilderError {
	#[error("Element(s) in flat blocks already taken when it was allocated to current block")]
	TakenElementMissing,
	#[error("Attempted to push elements on non-struct field")]
	InsertingIntoNonStruct,
	#[error("Unclaimed elements")]
	UnclaimedElements,
}

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
	pub fn from_flat_blocks(flat_blocks: &mut Vec<Option<FlatBlock>>) -> Result<Self, BlkBlockBuilderError> {
		let cloned = flat_blocks[0].clone().expect("Infallible");
		let ret = Self::from_flat_blocks_with_parent(flat_blocks, cloned)?;

		#[cfg(debug_assertions)]
		if flat_blocks.into_iter().all(|e|e.is_none()) == false {
			return Err(UnclaimedElements);
		}

		Ok(ret)
	}

	fn from_flat_blocks_with_parent(flat_blocks: &mut Vec<Option<FlatBlock>>, parent: FlatBlock) -> Result<Self, BlkBlockBuilderError> {
		let range = parent.location_range();
		let mut block = BlkField::Struct(parent.name, parent.fields);

		let block_range = flat_blocks[range].iter_mut().map(|e|e.take()).collect::<Option<Vec<FlatBlock>>>().ok_or(TakenElementMissing)?;

		for flat_block in block_range{
			block.insert_field(Self::from_flat_blocks_with_parent(flat_blocks, flat_block)?).ok_or(InsertingIntoNonStruct)?;
		}

		Ok(block)
	}
}
