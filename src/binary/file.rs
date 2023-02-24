pub enum FileType {
	BBF = 0x00,
	FAT = 0x01,
	FAT_ZST = 0x02,
	SLIM = 0x03,
	SLIM_ZSTD = 0x04,
	SLIM_ZST_DICT = 0x05,
}