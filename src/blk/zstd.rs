use std::io::{BufReader, Read};

use color_eyre::{eyre::ContextCompat, Report};
use zstd::{dict::DecoderDictionary, Decoder};

use crate::blk::file::FileType;

/// Decodes zstd compressed file using shared dictionary if available
pub fn decode_zstd(file_type: FileType, file: &[u8], frame_decoder: Option<&DecoderDictionary>) -> Result<Vec<u8>, Report> {
	let (len, to_decode) = if !file_type.is_slim() {
		let len_raw = &file[1..4];
		let len = u32::from_be_bytes([0, len_raw[2], len_raw[1], len_raw[0]]);
		let to_decode = &file[4..(len as usize + 4)];
		(len as usize, to_decode)
	} else {
		(file.len() - 1, &file[1..])
	};
	let mut out = Vec::with_capacity(len);

	let mut decoder = if file_type.needs_dict() {
		Decoder::with_prepared_dictionary(
			BufReader::new(&file[1..]),
			frame_decoder.context(format!(
				"File type: {file_type} marked as having dictionary, but none was passed"
			))?,
		)?
	} else {
		Decoder::new(to_decode)?
	};
	let _ = decoder.read_to_end(&mut out)?;
	Ok(out)
}

#[cfg(test)]
mod test {
	use std::{fs, io::Read};

	use zstd::{dict::DecoderDictionary, Decoder};
	use crate::blk::file::FileType;

	use crate::blk::zstd::decode_zstd;

	#[test]
	fn fat_zstd() {
		let decoded =
			decode_zstd(FileType::FAT_ZSTD, include_bytes!("../../samples/section_fat_zst.blk"), None).unwrap();
		pretty_assertions::assert_eq!(&decoded, &include_bytes!("../../samples/section_fat.blk"));
	}

	#[test]
	fn slim_zstd() {
		let decoded =
			decode_zstd(FileType::SLIM_ZSTD,include_bytes!("../../samples/section_slim_zst.blk"), None).unwrap();
		pretty_assertions::assert_eq!(
			&decoded,
			&include_bytes!("../../samples/section_slim.blk")[1..]
		) // Truncating the first byte, as it is magic byte for the SLIM format
	}

	#[test]
	fn slim_zstd_dict() {
		let file = fs::read("./samples/section_slim_zst_dict.blk").unwrap();
		let dict = fs::read(
			"./samples/bfb732560ad45234690acad246d7b14c2f25ad418a146e5e7ef68ba3386a315c.dict",
		)
		.unwrap();
		let frame_decoder = DecoderDictionary::copy(&dict);

		let mut decoder = Decoder::with_prepared_dictionary(&file[1..], &frame_decoder).unwrap();
		let mut out = vec![];
		decoder.read_to_end(&mut out).unwrap();
		pretty_assertions::assert_eq!(&out, &include_bytes!("../../samples/section_slim.blk")[1..])
		// Truncating the first byte, as it is magic byte for the SLIM format
	}
}
