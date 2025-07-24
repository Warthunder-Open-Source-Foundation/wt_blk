use std::ops::Range;

#[allow(unused_imports)] // Debug only imports make release output noisy
use crate::blk::blk_block_hierarchy::BlkBlockBuilderError::{
	InitialElementMissing,
	InsertingIntoNonStruct,
	TakenElementMissing,
	UnclaimedElements,
};
use crate::blk::blk_structure::BlkField;
use crate::blk::blk_string::BlkString;

#[derive(Debug, Clone, thiserror::Error, Eq, PartialEq)]
pub enum BlkBlockBuilderError {
	#[error("Element(s) in flat blocks already taken when it was allocated to current block")]
	TakenElementMissing,
	#[error("Attempted to push elements on non-struct field")]
	InsertingIntoNonStruct,
	#[error("Unclaimed elements")]
	UnclaimedElements,
	#[error("Initial element missing from flat blocks (length 0)")]
	InitialElementMissing,
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
	pub fn from_flat_blocks(
		flat_blocks: &mut Vec<Option<FlatBlock>>,
	) -> Result<Self, BlkBlockBuilderError> {
		let cloned = flat_blocks[0].take().ok_or(InitialElementMissing)?;
		let ret = Self::from_flat_blocks_with_parent(flat_blocks, cloned)?;

		#[cfg(debug_assertions)]
		if flat_blocks.into_iter().all(|e| e.is_none()) == false {
			return Err(UnclaimedElements);
		}

		Ok(ret)
	}

	fn from_flat_blocks_with_parent(
		flat_blocks: &mut Vec<Option<FlatBlock>>,
		parent: FlatBlock,
	) -> Result<Self, BlkBlockBuilderError> {
		let range = parent.location_range();
		let mut block = BlkField::Struct(parent.name, parent.fields);

		for flat_block in range {
			let new_parent = flat_blocks[flat_block].take().ok_or(TakenElementMissing)?;
			block
				.insert_field(Self::from_flat_blocks_with_parent(flat_blocks, new_parent)?)
				.ok_or(InsertingIntoNonStruct)?;
		}

		Ok(block)
	}
}
