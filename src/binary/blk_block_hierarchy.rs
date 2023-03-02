use std::ops::Range;
use crate::binary::blk_structure::BlkField;

pub struct FlatBlock {
	pub name: String,
	pub fields:  Vec<BlkField>,
	pub blocks: usize,
	pub offset: usize,
}

impl FlatBlock {
	fn location_range(&self) -> Range<usize> {
		self.offset..(self.offset + self.blocks)
	}
}

impl BlkField {
	pub fn from_flat_blocks(flat_blocks: &[FlatBlock]) -> Self {
		Self::from_flat_blocks_with_parent(flat_blocks, &flat_blocks[0])
	}

	fn from_flat_blocks_with_parent(flat_blocks: &[FlatBlock], parent: &FlatBlock) -> Self {
		let mut block = BlkField::Struct(parent.name.clone(), parent.fields.clone());

		for flat_block in &flat_blocks[parent.location_range()] {
			block.insert_field(Self::from_flat_blocks_with_parent(flat_blocks, flat_block)).unwrap();
		}

		block
	}
}