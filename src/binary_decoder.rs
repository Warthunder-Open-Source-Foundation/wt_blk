use std::{
	any::{type_name, Any},
	ops::Index,
};

use color_eyre::Report;

use crate::{
	binary_decoder::BinaryDecoderError::SeekingBackUnderflow,
	blk::{error::ParseError, leb128::uleb128, util::bytes_to_int},
};
use crate::blk::util::bytes_to_uint;

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
			.ok_or(SeekingBackUnderflow {
				cursor:   self.cursor,
				seekback: by,
			})
			.map_err(error_map)?;
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
			.ok_or(BinaryDecoderError::CursorOutOfBounds {
				buf_len: self.bytes.len(),
				cursor:  self.cursor,
			})
			.map_err(error_map)?;
		self.cursor += size_of::<u32>();
		Ok(bytes_to_uint(bytes).ok_or(integer_err::<u32>(bytes))?)
	}
}

/// Translates module local error to crate parent-module ParseError
fn error_map(e: BinaryDecoderError) -> ParseError {
	ParseError::BinaryDecoderError(e)
}

/// Builds error from invalid buffer and type
fn integer_err<T: Any>(buf: &[u8]) -> ParseError {
	ParseError::BinaryDecoderError(BinaryDecoderError::BadIntegerValue {
		buffer:    buf.to_vec(),
		type_name: type_name::<T>(),
	})
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
