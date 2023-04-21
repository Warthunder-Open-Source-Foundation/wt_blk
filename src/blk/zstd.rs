use std::{
	io::Read,
	ops::Deref,
	rc::Rc,
	sync::{Arc, Mutex},
	thread::sleep,
	time::Duration,
};

use zstd::{dict::DecoderDictionary, Decoder};

use crate::blk::file::FileType;

pub type BlkDecoder<'a> = DecoderDictionary<'a>;

pub fn decode_zstd(file: &[u8], mut frame_decoder: Arc<BlkDecoder>) -> Option<Vec<u8>> {
	// validate magic byte
	let file_type = FileType::from_byte(*file.get(0)?)?;

	let (len, mut to_decode) = if !file_type.is_slim() {
		let len_raw = &file[1..4];
		let len = u32::from_be_bytes([0, len_raw[2], len_raw[1], len_raw[0]]);
		let mut to_decode = &file[4..(len as usize + 4)];
		(len as usize, to_decode)
	} else {
		(file.len() - 1, &file[1..])
	};

	let decoded = if file_type.needs_dict() {
		let mut out = Vec::with_capacity(len);
		let mut decoder = Decoder::with_prepared_dictionary(&file[1..], &frame_decoder).unwrap();
		let _ = decoder.read_to_end(&mut out).ok()?;
		out
	} else {
		let mut out = Vec::with_capacity(len);
		let mut decoder = Decoder::with_prepared_dictionary(to_decode, &frame_decoder).unwrap();
		let _ = decoder.read_to_end(&mut out).ok()?;
		out
	};
	Some(decoded)
}

// UNUSED ATM
// pub fn decode_raw_zstd(file: &[u8], dict: Option<&[u8]> ) -> Option<Vec<u8>> {
// 	let len = file.len();
// 	Some(if let Some(dict) = dict {
// 		let mut frame_decoder = FrameDecoder::new();
// 		frame_decoder.add_dict(dict).expect("Dict should be available for dict file");
// 		let mut decoder = StreamingDecoder::new_with_decoder(&file[1..], frame_decoder).unwrap();
// 		let mut out = Vec::with_capacity(len);
// 		let _ = decoder.read_to_end(&mut out).ok()?;
// 		out
// 	} else {
// 		let mut file = file;
// 		let mut decoder =  StreamingDecoder::new(&mut file).ok()?;
// 		let mut out = Vec::with_capacity(len);
// 		let _ = decoder.read_to_end(&mut out).ok()?;
// 		out
// 	})
// }

pub fn eep() -> u8 {
	sleep(Duration::from_millis(1));
	42
}

#[cfg(test)]
mod test {
	use std::{fs, io, io::Read, rc::Rc, sync::Arc};

	use zstd::{dict::DecoderDictionary, Decoder};

	use crate::blk::zstd::decode_zstd;

	pub(crate) static DUMMY_DICT: fn() -> Arc<DecoderDictionary<'static>> =
		|| Arc::new(DecoderDictionary::copy(&[]));

	#[test]
	fn fat_zstd() {
		let decoded = decode_zstd(
			include_bytes!("../../samples/section_fat_zst.blk"),
			DUMMY_DICT(),
		)
		.unwrap();
		pretty_assertions::assert_eq!(&decoded, &include_bytes!("../../samples/section_fat.blk"));
	}

	#[test]
	fn slim_zstd() {
		let decoded = decode_zstd(
			include_bytes!("../../samples/section_slim_zst.blk"),
			DUMMY_DICT(),
		)
		.unwrap();
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
		let mut frame_decoder = DecoderDictionary::copy(&dict);

		let mut decoder = Decoder::with_prepared_dictionary(&file[1..], &frame_decoder).unwrap();
		let mut out = vec![];
		decoder.read_to_end(&mut out).unwrap();
		pretty_assertions::assert_eq!(&out, &include_bytes!("../../samples/section_slim.blk")[1..])
		// Truncating the first byte, as it is magic byte for the SLIM format
	}
}
