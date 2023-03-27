use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
	#[error("Empty buffer is not a valid ULEB var-int")]
	ZeroSizedUleb,

	#[error("Buffer ended prematurely, when current code-point expected continuation")]
	UnexpectedEndOfBufferUleb,
}