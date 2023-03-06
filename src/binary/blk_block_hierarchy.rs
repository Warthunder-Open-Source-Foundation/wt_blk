use std::ops::Range;
use std::rc::Rc;
use crate::binary::blk_structure::BlkField;
use crate::binary::blk_type::BlkCow;

#[derive(Debug, Clone)]
pub struct FlatBlock<'a> {
	pub name: BlkCow<'a>,
	pub fields:  Vec<BlkField<'a>>,
	pub blocks: usize,
	pub offset: usize,
}

impl FlatBlock<'_> {
	fn location_range(&self) -> Range<usize> {
		self.offset..(self.offset + self.blocks)
	}
}

impl <'a>BlkField<'a> {
	pub fn from_flat_blocks(flat_blocks: Vec<FlatBlock<'a>>) -> Self {
		let cloned = flat_blocks[0].clone();
		Self::from_flat_blocks_with_parent(&flat_blocks, cloned)
	}

	fn from_flat_blocks_with_parent(flat_blocks: &Vec<FlatBlock<'a>>, parent: FlatBlock<'a>) -> Self {
		let mut block = BlkField::Struct(parent.name.clone(), parent.fields.clone());

		for flat_block in &flat_blocks[parent.location_range()] {
			block.insert_field(Self::from_flat_blocks_with_parent(flat_blocks, flat_block.clone())).unwrap();
		}

		block
	}
}