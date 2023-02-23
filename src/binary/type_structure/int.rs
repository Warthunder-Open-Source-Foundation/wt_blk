use std::any::Any;
use crate::binary::type_structure::blk_type_trait::BlkType;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Int {
	inner: [u8; 4],
	value: u32,
}


impl BlkType for Int {
	fn from_bytes(info: [u8; 4], _: &[u8]) -> Self where Self: Sized {
		Self {
			inner: info,
			value: u32::from_le_bytes(info),
		}
	}


	fn get_any(&self) -> Box<dyn Any> {
		Box::new(self.value)
	}
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Int2 {
	value: [Int; 2],
}

impl BlkType for Int2 {
	fn from_bytes(info: [u8; 4], data_region: &[u8]) -> Self where Self: Sized {
		let offset = u32::from_le_bytes(info) as usize;
		let data = &data_region[offset..(offset + 8)];
		Self {
			value: [
				Int::from_bytes(data[0..4].try_into().unwrap(), &[]),
				Int::from_bytes(data[4..8].try_into().unwrap(), &[])
			],
		}
	}

	fn get_any(&self) -> Box<dyn Any> {
		Box::new(self.value)
	}
}

