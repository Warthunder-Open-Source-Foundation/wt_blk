use std::sync::Arc;
use pyo3::prelude::*;
use wt_blk::blk::DecoderDictionary;
use wt_blk::blk::nm_file::NameMap;

/// Formats the sum of two numbers as string.
#[pyfunction]
#[pyo3(signature = (blk, dict=None, nm=None))]
fn binary_blk_to_json(mut blk: Vec<u8>, dict: Option<Vec<u8>>, nm: Option<Vec<u8>>) -> PyResult<String> {
	let dict = dict.map(|d|{
		DecoderDictionary::copy(&d)
	});
	let nm = nm.map(|nm|{
		NameMap::from_encoded_file(&nm).map(|e|Arc::new(e))
	}).transpose().unwrap();
	let blk = wt_blk::blk::unpack_blk(&mut blk, dict.as_ref(), nm).unwrap();
	Ok(blk.as_serde_json_string().unwrap())
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn wt_blk_pybindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
	m.add_function(wrap_pyfunction!(binary_blk_to_json, m)?)?;
	Ok(())
}