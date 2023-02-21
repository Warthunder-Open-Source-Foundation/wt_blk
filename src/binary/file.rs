pub enum FileType {
	BBF = 0x00,
	FAT = 0x01,
	FAT_ZST = 0x02,
	SLIM = 0x03,
	SLIM_ZSTD = 0x04,
	SLIM_ZST_DICT = 0x05,
}

#[derive(Debug)]
pub struct FatBLk {
	pub names_count: usize,
	pub names_data_size: usize,
	pub names: Vec<String>,
	pub blocks_count: usize,
	pub params_count: usize,
	pub params_data_size: usize,
}