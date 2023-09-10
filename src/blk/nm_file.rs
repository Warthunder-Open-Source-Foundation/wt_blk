use std::{io::Read, sync::Arc};

use color_eyre::{eyre::ContextCompat, Report};
use zstd::Decoder;

use crate::blk::{blk_type::BlkString, leb128::uleb128};

#[derive(Clone, Debug)]
pub struct NameMap {
	pub binary: Vec<u8>,
	pub parsed: Arc<Vec<BlkString>>,
}

impl NameMap {
	pub fn idx_parsed(&self, idx: usize) -> Option<&BlkString> {
		self.parsed.get(idx)
	}

	pub fn from_encoded_file(file: &[u8]) -> Result<Self, Report> {
		let decoded = Self::decode_nm_file(file)?;

		let names = Self::parse_slim_nm(&decoded);

		Ok(Self {
			parsed: Arc::new(names),
			binary: decoded,
		})
	}

	pub fn decode_nm_file(file: &[u8]) -> Result<Vec<u8>, Report> {
		let _names_digest = &file.get(0..8).context(format!(
			"File out of bounds for range 0..8, found len: {}",
			file.len()
		))?;
		let _dict_digest = &file[8..40];
		let mut zstd_stream = &file[40..];
		let mut decoder = Decoder::new(&mut zstd_stream)?;
		let mut out = Vec::with_capacity(file.len());
		let _ = decoder.read_to_end(&mut out)?;
		Ok(out)
	}

	pub fn parse_name_section(file: &[u8]) -> Vec<BlkString> {
		let mut start = 0_usize;
		let mut names = vec![];
		for (i, val) in file.iter().enumerate() {
			if *val == 0 {
				names.push(Arc::from(
					String::from_utf8_lossy(&file[start..i]),
				));
				start = i + 1;
			}
		}
		names
	}

	pub fn parse_slim_nm(name_map: &[u8]) -> Vec<BlkString> {
		let mut nm_ptr = 0;

		let (offset, names_count) = uleb128(&name_map[nm_ptr..]).unwrap();
		nm_ptr += offset;

		let (offset, names_data_size) = uleb128(&name_map[nm_ptr..]).unwrap();
		nm_ptr += offset;

		let names = NameMap::parse_name_section(&name_map[nm_ptr..(nm_ptr + names_data_size)]);

		if names_count != names.len() {
			panic!("Should be equal"); // TODO: Change to result when fn signature allows for it
		}

		names
	}
}

#[cfg(test)]
mod test {
	use std::fs;

	use crate::blk::{leb128::uleb128, nm_file::NameMap};

	#[test]
	fn test_any_stream() {
		let decoded = NameMap::parse_name_section("a\0b\0c\0".as_bytes());
		assert_eq!(
			vec!["a", "b", "c"],
			decoded
				.iter()
				.map(|x| x.to_string())
				.collect::<Vec<String>>()
		)
	}

	#[test]
	fn test_nm_file() {
		let file = fs::read("./samples/nm").unwrap();
		let decoded = NameMap::decode_nm_file(&file).unwrap();
		assert_eq!(&fs::read("./samples/names").unwrap(), &decoded)
	}

	#[test]
	fn nm_parity() {
		let nm = fs::read("../wt_blk/samples/rendist/nm").unwrap();
		let nm = NameMap::decode_nm_file(&nm).unwrap();

		let mut nm_ptr = 0;

		let (offset, _names_count) = uleb128(&nm[nm_ptr..]).unwrap();
		nm_ptr += offset;

		let (offset, names_data_size) = uleb128(&nm[nm_ptr..]).unwrap();
		nm_ptr += offset;

		let _old = {
			let mut buff = vec![];
			let mut names = vec![];
			for val in &nm[nm_ptr..(nm_ptr + names_data_size)] {
				if *val == 0 {
					if let Ok(good) = String::from_utf8(buff.clone()) {
						names.push(good);
					} else {
						println!("{:?}", String::from_utf8_lossy(&buff));
					}
					buff.clear();
				} else {
					buff.push(*val);
				}
			}
			names
		};
		// let new = {
		// 	nm[nm_ptr..(nm_ptr + names_data_size)].split(|b| *b == 0).map(|bs| String::from_utf8_lossy(bs).to_string()).collect::<Vec<_>>()
		// };
		// assert_eq!(old, new);
	}
}
