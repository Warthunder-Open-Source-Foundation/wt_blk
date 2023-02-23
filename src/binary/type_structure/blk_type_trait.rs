use std::any::Any;
use std::fmt::Debug;
use std::num::NonZeroU8;

pub trait BlkType: Debug  {
	/// Info points contains/points at the values
	/// Items that fit into 32 bits are directly stored in the info
	/// Items larger than 32 bits are represented as an offset in the data-region
	fn from_bytes(info: [u8; 4], data_region: &[u8]) -> Self where Self: Sized;

	/// Returns dynamic reference to type, needs downcasting to reach actual content
	/// TODO: Not sure if this is smart at all
	fn get_any(&self) -> Box<dyn Any>;
}