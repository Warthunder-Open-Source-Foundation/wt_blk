use iex::iex;
use crate::blk::error::BlkError;

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, strum::Display)]
// BLK file type, always first byte of file
pub enum FileType {
	/// Unknown and unsupported
	BBF           = 0x00,
	/// BLK file with integrated name-map, no external map is required
	FAT           = 0x01,
	/// Same as FAT but with ZSTD compression
	FAT_ZSTD      = 0x02,
	/// Has name map externally stored
	SLIM          = 0x03,
	/// Same as SLIM but ZSTD compressed
	SLIM_ZSTD     = 0x04,
	/// Same as SLIM_ZSTD, but with a ZSTD dictionary
	SLIM_ZST_DICT = 0x05,
}

impl FileType {
	#[iex]
	pub fn from_byte(input: u8) -> Result<Self, BlkError> {
		match input {
			0x00 => Ok(Self::BBF),
			0x01 => Ok(Self::FAT),
			0x02 => Ok(Self::FAT_ZSTD),
			0x03 => Ok(Self::SLIM),
			0x04 => Ok(Self::SLIM_ZSTD),
			0x05 => Ok(Self::SLIM_ZST_DICT),
			_ => Err(&*(format!("unrecognized header: {input:x}").leak())),
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
