use std::io::Read;
use ruzstd::StreamingDecoder;

pub fn decode_zstd(file: &[u8]) -> Option<Vec<u8>> {
	// validate magic byte
	if file[0] != 0x2 {
		return None;
	}

	let len_raw = &file[1..4];
	let len = u32::from_be_bytes([
		0,
		len_raw[2],
		len_raw[1],
		len_raw[0],
	]);
	let mut to_decode = &file[4..(len as usize + 4)];
	let mut decoder = StreamingDecoder::new(&mut to_decode).ok()?;
	let mut out = Vec::with_capacity(len as usize);
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
		assert_eq!(&decoded, include_bytes!("../../samples/section_fat.blk"))
	}
}