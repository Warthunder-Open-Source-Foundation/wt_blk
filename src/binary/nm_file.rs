use std::io::Read;
use ruzstd::StreamingDecoder;

pub fn decode_nm_file(file: &[u8]) -> Option<Vec<u8>> {
	let _names_digest = &file[0..8];
	let _dict_digest = &file[8..40];
	let mut zstd_stream = &file[40..];
	let mut decoder = StreamingDecoder::new(&mut zstd_stream).ok()?;
	let mut out = Vec::with_capacity(file.len());
	let _ = decoder.read_to_end(&mut out).ok()?;
	Some(out)
}

pub fn parse_name_section(file: &[u8]) -> Vec<String> {
	let mut buff = vec![];
	let mut names = vec![];
	for val in file {
		if *val == 0 {
			names.push(String::from_utf8(buff.clone()).unwrap());
			buff.clear();
		} else {
			buff.push(*val);
		}
	}
	names
}

#[cfg(test)]
mod test {
	use std::fs;
	use crate::binary::nm_file::decode_nm_file;

	#[test]
	fn test_nm_file() {
		let file = fs::read("./samples/nm").unwrap();
		let decoded = decode_nm_file(&file).unwrap();
		assert_eq!(&fs::read("./samples/names").unwrap(), &decoded)
	}
}