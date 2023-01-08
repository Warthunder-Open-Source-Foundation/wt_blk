pub mod error;

use serde_json::Value;
use crate::error::WTBlkError;

#[derive(Debug)]
pub struct WTBlk {
	inner: Value,
}


impl WTBlk {
	pub fn new(str: &str) -> Result<Self, WTBlkError> {
		let inner: Value = serde_json::from_str(str)?;
		Ok(Self {
			inner,
		})
	}
	pub fn pointer<'a>(&'a self, pointer: &'a str) -> Result<&Value, WTBlkError> {
		self.inner.pointer(pointer).ok_or_else(||WTBlkError::NoSuchValue(pointer))
	}
}
