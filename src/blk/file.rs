use crate::blk::error::ParseError;

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, strum::Display)]
// BLK file type, always first byte of file
pub enum FileType {
	BBF           = 0x00,
	FAT           = 0x01,
	FAT_ZSTD      = 0x02,
	SLIM          = 0x03,
	SLIM_ZSTD     = 0x04,
	SLIM_ZST_DICT = 0x05,
}

impl FileType {
	pub fn from_byte(input: u8) -> Result<Self, ParseError> {
		match input {
			0x00 => Ok(Self::BBF),
			0x01 => Ok(Self::FAT),
			0x02 => Ok(Self::FAT_ZSTD),
			0x03 => Ok(Self::SLIM),
			0x04 => Ok(Self::SLIM_ZSTD),
			0x05 => Ok(Self::SLIM_ZST_DICT),
			_ => Err(ParseError::UnrecognizedBlkHeader { header: input }),
		}
	}

	pub fn is_slim(&self) -> bool {
		match self {
			FileType::SLIM => true,
			FileType::SLIM_ZSTD => true,
			FileType::SLIM_ZST_DICT => true,
			_ => false,
		}
	}

	pub fn is_zstd(&self) -> bool {
		match self {
			FileType::FAT_ZSTD => true,
			FileType::SLIM_ZSTD => true,
			FileType::SLIM_ZST_DICT => true,
			_ => false,
		}
	}

	pub fn needs_dict(&self) -> bool {
		*self == FileType::SLIM_ZST_DICT
	}
}
