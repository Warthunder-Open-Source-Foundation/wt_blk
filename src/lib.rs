pub mod error;

use serde_json::{Map, Value};
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
		self.inner.pointer(pointer).ok_or_else(||WTBlkError::NoSuchValue(pointer.to_owned()))
	}
	pub fn inner(&mut self) -> &mut Value {
		&mut self.inner
	}

	// Useful for direct field accessing
	pub fn bool<'a>(&'a self, pointer: &'a str) -> Result<bool, WTBlkError> {
		self.pointer(pointer)?.as_bool().ok_or_else(||WTBlkError::Parse(pointer.to_owned()))
	}
	pub fn float<'a>(&'a self, pointer: &'a str) -> Result<f64, WTBlkError> {
		self.pointer(pointer)?.as_f64().ok_or_else(||WTBlkError::Parse(pointer.to_owned()))
	}
	pub fn int<'a>(&'a self, pointer: &'a str) -> Result<i64, WTBlkError> {
		self.pointer(pointer)?.as_i64().ok_or_else(||WTBlkError::Parse(pointer.to_owned()))
	}
	pub fn str<'a>(&'a self, pointer: &'a str) -> Result<&str, WTBlkError> {
		self.pointer(pointer)?.as_str().ok_or_else(||WTBlkError::Parse(pointer.to_owned()))
	}
	pub fn objects<'a>(&'a self, pointer: &'a str) -> Result<&Map<String, Value>, WTBlkError> {
		self.pointer(pointer)?.as_object().ok_or_else(||WTBlkError::Parse(pointer.to_owned()))
	}
	pub fn array<'a>(&'a self, pointer: &'a str) -> Result<&Vec<Value>, WTBlkError> {
		self.pointer(pointer)?.as_array().ok_or_else(||WTBlkError::Parse(pointer.to_owned()))
	}
}
