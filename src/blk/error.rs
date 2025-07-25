use std::{ops::Range, string::FromUtf8Error};

use thiserror::Error;

use crate::blk::blk_block_hierarchy::BlkBlockBuilderError;

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum ParseError {
	#[error("Empty buffer is not a valid ULEB var-int")]
	ZeroSizedUleb,

	#[error("Buffer ended prematurely, when current code-point expected continuation")]
	UnexpectedEndOfBufferUleb,

	#[error("Indexing into the data region was unsuccessful, most likely due to an invalid ULEB offset stemming from bad offsets")]
	// Offset into buffer used
	DataRegionBoundsExceeded(Range<usize>),

	// NOTE: This should not really occur, as the ptr should go out of bounds much earlier if an offset is bad
	#[error("Residual buffer for block information was out of bounds")]
	ResidualBlockBuffer,

	#[error("Blk value parsing failed")]
	BadBlkValue,

	#[error("Attempted to parse SLIM blk file without a NN")]
	SlimBlkWithoutNm,

	#[error("Invalid BLK header: {header:X}")]
	UnrecognizedBlkHeader { header: u8 },

	#[error("Dictionary was invalid")]
	InvalidDict {},

	#[error("Missing dictionary")]
	MissingDict {},

	#[error(transparent)]
	BlkBlockBuilderError(BlkBlockBuilderError),

	#[error(transparent)]
	Utf8Error(#[from] FromUtf8Error),

	#[error("Custom: {0}")]
	Custom(String),
}
