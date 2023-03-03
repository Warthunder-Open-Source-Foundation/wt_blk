use std::fs;
use std::fs::ReadDir;
use std::io::{stdout, Write};
use crate::binary::file::FileType;
use crate::binary::parser::parse_blk;
use crate::binary::zstd::decode_zstd;

#[cfg(test)]
mod test {
	use std::fs;
	use std::fs::ReadDir;
	use std::mem::size_of;
	use std::path::Path;
	use crate::binary::blk_type::BlkType;
	use crate::binary::leb128::uleb128;
	use crate::binary::nm_file::{decode_nm_file, parse_name_section};
	use crate::binary::parser::parse_blk;
	use crate::binary::test::test_parse_dir;
	use crate::binary::zstd::{decode_raw_zstd, decode_zstd};

	#[test]
	fn fat_blk() {
		let file = fs::read("./samples/section_fat.blk").unwrap();
		let output = parse_blk(&file, true, false,None);
	}

	#[test]
	fn fat_blk_router_probe() {
		let file = fs::read("./samples/route_prober.blk").unwrap();
		let output = parse_blk(&file, false, false,None);

	}

	#[test]
	fn slim_blk() {
		let file = fs::read("./samples/section_slim.blk").unwrap();
		let nm = fs::read("./samples/names").unwrap();
		let output = parse_blk(&file, true, true,Some(&nm));
	}

	#[test]
	fn test_all() {
		let nm = fs::read("./samples/vromfs/aces.vromfs.bin_u/nm").unwrap();
		let dict = fs::read("./samples/vromfs/aces.vromfs.bin_u/ca35013aabca60792d5203b0137d0a8720d1dc151897eb856b12318891d08466.dict").unwrap();
		let nm = decode_nm_file(&nm).unwrap();

		let dir: ReadDir = fs::read_dir("./samples/vromfs/aces.vromfs.bin_u").unwrap();
		let mut total = 0;
		test_parse_dir(dir, &mut total, &dict, &nm);
		println!("Successfully parsed {} files! Thats all of them.", total);
		panic!("bonk")
	}
}

fn test_parse_dir(dir: ReadDir, total: &mut i32, dict: &[u8], nm: &[u8]) {
	for file in dir {
		let file = file.unwrap();
		if file.metadata().unwrap().is_dir() {
			test_parse_dir(file.path().read_dir().unwrap(), total, dict, nm);
		} else {
			let fname =  file.file_name().to_str().unwrap().to_owned();
			if fname.ends_with(".blk") {
				let mut read = fs::read(file.path()).unwrap();
				let mut offset = 0;
				if let Some(file_type) = FileType::from_byte(read[0]) {
					if file_type.is_zstd() {
						read = decode_zstd(&read, Some(dict)).unwrap();
					} else {
						// uncompressed Slim and Fat files retain their initial magic bytes
						offset = 1;
					}


					parse_blk(&read[offset..], false, file_type.is_slim(), Some(nm));
				} else {
					println!("Skipped {} as it was plaintext", fname);
				}
				*total += 1;
				eprintln!("{}", total);
			}
		}
	}
}