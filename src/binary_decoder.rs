use std::ops::Index;
use crate::binary_decoder::BinaryDecoderError::SeekingBackUnderflow;

use crate::blk::{
	error::ParseError,
	leb128::uleb128,
};

type BinaryDecoderResult<T> = Result<T, ParseError>;

pub struct BinaryDecoder<'a> {
	bytes:  &'a [u8],
	cursor: usize,
}

impl<'a> BinaryDecoder<'a> {
	pub fn new(bytes: &'a [u8]) -> Self {
		Self { bytes, cursor: 0 }
	}

	/// Returns the next bytes at the current cursor
	pub fn next_bytes(&self) -> BinaryDecoderResult<&'a [u8]> {
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
			.ok_or(SeekingBackUnderflow {
				cursor:   self.cursor,
				seekback: by,
			})
			.map_err(error_map)?;
		Ok(())
	}

	/// Returns next uleb encoded integer, advancing the cursor
	pub fn next_uleb(&mut self) -> BinaryDecoderResult<usize> {
		let bytes = self.next_bytes()?;
		let (uleb_len, value) = uleb128(bytes)?;
		self.seek(uleb_len);
		Ok(value)
	}
}

/// Translates module local error to crate parent-module ParseError
fn error_map(e: BinaryDecoderError) -> ParseError {
	ParseError::BinaryDecoderError(e)
}

#[derive(Clone, thiserror::Error, Debug, PartialEq, Eq)]
pub enum BinaryDecoderError {
	#[error("Cursor at position {cursor} is out of bounds for {buf_len}")]
	CursorOutOfBounds { buf_len: usize, cursor: usize },
	#[error(
		"Failed to seek backwards because seekback {seekback} was greater than cursor {cursor}"
	)]
	SeekingBackUnderflow { cursor: usize, seekback: usize },
}
