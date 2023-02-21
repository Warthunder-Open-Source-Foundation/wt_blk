use std::io::Read;
use ruzstd::StreamingDecoder;

pub fn decode_zstd(file: &[u8]) -> Option<Vec<u8>> {
	// validate magic byte
	let is_fat = match file.get(0)? {
		0x2 => true, // FAT_ZST
		0x4 | 0x5 => false, // SLIM_ZST and SLIM_ZST_DICT
		_ => return None
	};

	let (len, mut to_decode) = if is_fat {
		let len_raw = &file[1..4];
		let len = u32::from_be_bytes([
			0,
			len_raw[2],
			len_raw[1],
			len_raw[0],
		]);
		let mut to_decode = &file[4..(len as usize + 4)];
		(len as usize, to_decode)
	} else {
		(file.len() - 1, &file[1..])
	};

	let mut decoder = StreamingDecoder::new(&mut to_decode).ok()?;
	let mut out = Vec::with_capacity(len);
	let _ = decoder.read_to_end(&mut out).ok()?;
	Some(out)
}

#[cfg(test)]
mod test {
	use std::fs;
	use crate::binary::zstd::decode_zstd;

	#[test]
	fn fat_zstd() {
		let decoded = decode_zstd(include_bytes!("../../samples/section_fat_zst.blk")).unwrap();
		assert_eq!(&decoded, &include_bytes!("../../samples/section_fat.blk"))
	}

	#[test]
	fn slim_zstd() {
		let decoded = decode_zstd(include_bytes!("../../samples/section_slim_zst.blk")).unwrap();
		assert_eq!(&decoded, &include_bytes!("../../samples/section_slim.blk")[1..]) // Truncating the first byte, as it is magic byte for the SLIM format
	}

	// TODO: Fix decoding failure with dict mode
	// #[test]
	// fn slim_zstd_dict() {
	// 	let decoded = decode_zstd(include_bytes!("../../samples/section_slim_zst_dict.blk")).unwrap();
	// 	assert_eq!(&decoded, &include_bytes!("../../samples/section_slim.blk")[1..]) // Truncating the first byte, as it is magic byte for the SLIM format
	// }
}