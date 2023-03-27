use std::ops::{Range, RangeInclusive};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ParseError {
	#[error("Empty buffer is not a valid ULEB var-int")]
	ZeroSizedUleb,

	#[error("Buffer ended prematurely, when current code-point expected continuation")]
	UnexpectedEndOfBufferUleb,

	#[error("Indexing into the data region was unsuccessful, most likely due to an invalid ULEB offset stemming from bad offsets")]
	// Offset into buffer used
	DataRegionBoundsExceeded(Range<usize>),
}