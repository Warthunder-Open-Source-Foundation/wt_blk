use std::io::Read;
use std::rc::Rc;
use zstd::Decoder;
use crate::binary::blk_type::BlkCow;
use crate::binary::leb128::uleb128;


pub fn parse_slim_nm(name_map: &[u8]) -> Vec<BlkCow> {
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

#[derive(Clone, Debug)]
pub struct NameMap<'a> {
	binary: Vec<u8>,
	parsed: Vec<BlkCow<'a>>,
}

impl <'a>NameMap<'a> {
	pub fn from_encoded_file(file: &'a [u8]) -> Option<Self> {
		let decoded = Self::decode_nm_file(file)?;

		let mut start = 0_usize;
		let mut names = vec![];
		for (i, val) in file.iter().enumerate() {
			if *val == 0 {
				names.push(String::from_utf8_lossy(&file[start..i]));
				start = i;
			}
		}

		Some(Self {
			parsed: names,
			binary: decoded,
		})
	}

	pub fn decode_nm_file(file: &[u8]) -> Option<Vec<u8>> {
		let _names_digest = &file[0..8];
		let _dict_digest = &file[8..40];
		let mut zstd_stream = &file[40..];
		let mut decoder = Decoder::new(&mut zstd_stream).ok()?;
		let mut out = Vec::with_capacity(file.len());
		let _ = decoder.read_to_end(&mut out).ok()?;
		Some(out)
	}

	pub fn parse_name_section(file: &[u8]) -> Vec<BlkCow> {
		let mut start = 0_usize;
		let mut names = vec![];
		for (i, val) in file.iter().enumerate() {
			if *val == 0 {
				names.push(String::from_utf8_lossy(&file[start..i]));
				start = i +1;
			}
		}
		names
	}
}

#[cfg(test)]
mod test {
	use std::fs;
	use crate::binary::leb128::uleb128;
	use crate::binary::nm_file::NameMap;

	#[test]
	fn test_any_stream() {
		let decoded = NameMap::parse_name_section("a\0b\0c\0".as_bytes());
		assert_eq!(vec!["a", "b", "c"], decoded.iter().map( |x|x.to_string()).collect::<Vec<String>>())
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

		let (offset, names_count) = uleb128(&nm[nm_ptr..]).unwrap();
		nm_ptr += offset;

		let (offset, names_data_size) = uleb128(&nm[nm_ptr..]).unwrap();
		nm_ptr += offset;


		let old = {
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