use std::{
	any::{type_name, Any},
	mem::size_of,
	ops::Index,
};

use color_eyre::Report;

use crate::{
	binary_decoder::BinaryDecoderError::SeekingBackUnderflow,
	blk::{
		leb128::uleb128,
		util::{bytes_to_int, bytes_to_uint},
	},
};
use crate::blk::error::BlkError;

type BinaryDecoderResult<T> = Result<T, BlkError>;

pub struct BinaryDecoder<'a> {
	bytes:  &'a [u8],
	cursor: usize,
}

impl<'a> BinaryDecoder<'a> {
	pub fn new(bytes: &'a [u8]) -> Self {
		Self { bytes, cursor: 0 }
	}

	/// Returns the next bytes at the current cursor
	pub fn remaining_bytes(&self) -> BinaryDecoderResult<&'a [u8]> {
		if self.cursor < self.bytes.len() {
			Ok(self.bytes.index(self.cursor..))
		} else {
			Err(error_map(BinaryDecoderError::CursorOutOfBounds {
				buf_len: self.bytes.len(),
				cursor:  self.cursor,
			}))
		}
	}

	/// Unchecked forwards seeking operation
	pub fn seek(&mut self, by: usize) {
		self.cursor += by
	}

	/// Checked backwards-seeking operation
	pub fn seek_back(&mut self, by: usize) -> BinaryDecoderResult<()> {
		self.cursor = self
			.cursor
			.checked_add(by)
			.ok_or("Failed to add to cursor")?;
		Ok(())
	}

	/// Returns next uleb encoded integer, advancing the cursor
	pub fn next_uleb(&mut self) -> BinaryDecoderResult<usize> {
		let bytes = self.remaining_bytes()?;
		let (uleb_len, value) = uleb128(bytes)?;
		self.seek(uleb_len);
		Ok(value)
	}

	pub fn next_u32(&mut self) -> BinaryDecoderResult<u32> {
		let bytes = self
			.bytes
			.get(self.cursor..(self.cursor + size_of::<u32>()))
			.ok_or("integer out of bounds for next u32")?;
		self.cursor += size_of::<u32>();
		Ok(bytes_to_uint(bytes)?)
	}
}

#[derive(Clone, thiserror::Error, Debug, PartialEq, Eq)]
pub enum BinaryDecoderError {
	#[error("Cursor at position {cursor} is out of bounds for {buf_len}")]
	CursorOutOfBounds { buf_len: usize, cursor: usize },
	#[error(
		"Failed to seek backwards because seekback {seekback} was greater than cursor {cursor}"
	)]
	SeekingBackUnderflow { cursor: usize, seekback: usize },
	#[error("Failed to construct {type_name} from {buffer:?}")]
	BadIntegerValue {
		buffer:    Vec<u8>,
		type_name: &'static str,
	},
}
