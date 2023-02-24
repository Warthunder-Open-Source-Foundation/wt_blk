
#[repr(u8)]
pub enum FileType {
	BBF = 0x00,
	FAT = 0x01,
	FAT_ZSTD = 0x02,
	SLIM = 0x03,
	SLIM_ZSTD = 0x04,
	SLIM_ZST_DICT = 0x05,
}

impl FileType {
	pub fn from_byte(input: u8) -> Option<Self> {
		match input {
			0x00 => Some(Self::BBF),
			0x01 =>Some( Self::FAT),
			0x02 => Some(Self::FAT_ZSTD),
			0x03 => Some(Self::SLIM),
			0x04 => Some(Self::SLIM_ZSTD),
			0x05 => Some(Self::SLIM_ZST_DICT),
			_ => {None}
		}
	}
}