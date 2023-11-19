use std::ops::Index;
use crate::blk::blk_decoder::BlkDecoderError::SeekingBackUnderflow;
use crate::blk::error::ParseError;
use crate::blk::leb128::uleb128;

type BlkResult<T> = Result<T, ParseError>;

pub struct BlkDecoder<'a> {
	bytes: &'a [u8],
	cursor: usize,
}

impl <'a>BlkDecoder<'a> {
	pub fn new(bytes: &'a [u8]) -> Self {
		Self {
			bytes,
			cursor: 0,
		}
	}

	/// Returns the next bytes at the current cursor
	pub fn next_bytes(&self) -> BlkResult<&'a [u8]> {
		if self.cursor < self.bytes.len() {
			Ok(self.bytes.index(self.cursor..))
		} else {
			Err(error_map(BlkDecoderError::CursorOutOfBounds {
				buf_len: self.bytes.len(),
				cursor: self.cursor,
			}))
		}
	}

	/// Unchecked forwards seeking operation
	pub fn seek(&mut self, by: usize) {
		self.cursor += by
	}

	/// Checked backwards-seeking operation
	pub fn seek_back(&mut self, by: usize) -> BlkResult<()> {
		self.cursor = self.cursor.checked_add(by)
			.ok_or(SeekingBackUnderflow { cursor: self.cursor, seekback: by })
			.map_err(error_map)?;
		Ok(())
	}

	/// Returns next uleb encoded integer, advancing the cursor
	pub fn next_uleb(&mut self) -> BlkResult<usize> {
		let bytes = self.next_bytes()?;
		let (uleb_len, value) = uleb128(bytes)?;
		self.seek(uleb_len);
		Ok(value)
	}
}

/// Translates module local error to crate parent-module ParseError
fn error_map(e: BlkDecoderError) -> ParseError {
	ParseError::BlkDecoderError(e)
}

#[derive(Clone, thiserror::Error, Debug, PartialEq, Eq)]
pub enum BlkDecoderError {
	#[error("Cursor at position {cursor} is out of bounds for {buf_len}")]
	CursorOutOfBounds {
		buf_len: usize,
		cursor: usize,
	},
	#[error("Failed to seek backwards because seekback {seekback} was greater than cursor {cursor}")]
	SeekingBackUnderflow {
		cursor: usize,
		seekback: usize,
	}
}