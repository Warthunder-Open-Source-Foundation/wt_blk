use std::sync::Arc;
use wasm_bindgen::prelude::wasm_bindgen;
use wt_blk::blk::DecoderDictionary;
use wt_blk::blk::nm_file::NameMap;

/// Converts binary BLK into json string
#[wasm_bindgen]
pub fn blk_to_json(mut blk: Vec<u8>, dict: Option<Vec<u8>>, nm: Option<Vec<u8>>) -> String {
	let dict = dict.map(|d|{
		DecoderDictionary::copy(&d)
	});
	let nm = nm.map(|nm|{
		NameMap::from_encoded_file(&nm).map(|e|Arc::new(e))
	}).transpose().unwrap();
	let blk = wt_blk::blk::unpack_blk(&mut blk, dict.as_ref(), nm).unwrap();
	blk.merge_fields();
	blk.as_serde_json_string().unwrap()
}