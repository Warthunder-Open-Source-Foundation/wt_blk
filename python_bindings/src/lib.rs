use std::sync::Arc;
use pyo3::prelude::*;
use wt_blk::blk::DecoderDictionary;
use wt_blk::blk::name_map::NameMap;

/// Deserialise a binary BLK file into a JSON string
#[pyfunction]
#[pyo3(signature = (blk, dict=None, nm=None))]
fn binary_blk_to_json(mut blk: Vec<u8>, dict: Option<Vec<u8>>, nm: Option<Vec<u8>>) -> PyResult<String> {
	let dict = dict.map(|d|{
		DecoderDictionary::copy(&d)
	});
	let nm = nm.map(|nm|{
		NameMap::from_encoded_file(&nm).map(|e|Arc::new(e))
	}).transpose().unwrap();
	let mut blk = wt_blk::blk::unpack_blk(&mut blk, dict.as_ref(), nm).unwrap();
	blk.merge_fields();
	Ok(blk.as_serde_json_string().unwrap())
}

#[pymodule]
fn wt_blk_pybindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
	m.add_function(wrap_pyfunction!(binary_blk_to_json, m)?)?;
	Ok(())
}