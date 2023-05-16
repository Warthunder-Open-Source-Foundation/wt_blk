use std::ffi::OsStr;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use zstd::dict::DecoderDictionary;

use crate::blk::blk_structure::BlkField;
use crate::blk::BlkOutputFormat;
use crate::blk::file::FileType;
use crate::blk::nm_file::NameMap;
use crate::blk::output_formatting_conf::FormattingConfiguration;
use crate::blk::parser::parse_blk;
use crate::blk::zstd::decode_zstd;
use crate::vromf::binary_container::decode_bin_vromf;
use crate::vromf::error::VromfError;
use crate::vromf::inner_container::decode_inner_vromf;
use crate::vromf::util::path_stringify;
use crate::vromf::VromfError::MissingDict;

pub type File = (PathBuf, Vec<u8>);

#[derive()]
struct DictWrapper<'a>(DecoderDictionary<'a>);

impl Debug for DictWrapper<'_> { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "DecoderDictionary {{...}}") } }

#[derive(Debug)]
pub struct VromfUnpacker<'a> {
	files: Vec<File>,
	dict: Option<Arc<DictWrapper<'a>>>,
	nm: Option<Arc<NameMap>>,
}

impl VromfUnpacker<'_> {
	pub fn from_file(file: File) -> Result<Self, VromfError> {
		let decoded = decode_bin_vromf(&file.1)?;
		let inner = decode_inner_vromf(&decoded)?;

		let nm = inner.iter()
			.find(|elem| elem.0.file_name() == Some(OsStr::new("nm")))
			.map(|elem|
				NameMap::from_encoded_file(&elem.1).ok_or(VromfError::InvalidNm)
			)
			.transpose()?
			.map(|elem| Arc::new(elem));

		let dict = inner.iter()
			.find(|elem| elem.0.extension() == Some(OsStr::new("dict")))
			.map(|elem|
				Arc::new(DictWrapper(DecoderDictionary::copy(&elem.1)))
			);

		Ok(Self {
			files: inner,
			dict,
			nm,
		})
	}

	pub fn unpack_all(
		self,
		unpack_blk_into: Option<BlkOutputFormat>,
	) -> Result<Vec<File>, VromfError> {
		self.files.into_iter()
			.map(|mut file| {
				match () {
					_ if maybe_blk(&file) => {
						if let Some(format) = unpack_blk_into {
							let mut offset = 0;
							let file_type = FileType::from_byte(file.1[0])?;
							if file_type.is_zstd() {
								file.1 = decode_zstd(&file.1, self.dict.as_ref().map(|e| &e.0)).unwrap();
							} else {
								// uncompressed Slim and Fat files retain their initial magic bytes
								offset = 1;
							};

							let parsed = parse_blk(&file.1[offset..], file_type.is_slim(), self.nm.clone())?;
							match format {
								BlkOutputFormat::Json(config) => {
									file.1 = parsed.as_ref_json(config)?.into_bytes();
								}
								BlkOutputFormat::BlkText => {
									file.1 = parsed.as_blk_text().into_bytes();
								}
							}
						}
						Ok(file)
					}

					// Default to the raw file
					_ => {
						Ok(file)
					}
				}
			}).collect::<Result<Vec<File>, VromfError>>()
	}

	pub fn unpack_one(&self, path_name: &Path, unpack_blk_into: Option<BlkOutputFormat>)-> Result<Vec<u8>, VromfError> {
		let mut file = self.files.iter().find(|e|e.0 == path_name).ok_or(VromfError::FileNotInVromf { path_name: path_name.to_str().unwrap_or("PATH PARSE FAILED, REPORT THIS ERROR").to_string()})?.to_owned();
		match () {
			_ if maybe_blk(&file) => {
				if let Some(format) = unpack_blk_into {
					let mut offset = 0;
					let file_type = FileType::from_byte(file.1[0])?;
					if file_type.is_zstd() {
						file.1 = decode_zstd(&file.1, self.dict.as_ref().map(|e| &e.0)).unwrap();
					} else {
						// uncompressed Slim and Fat files retain their initial magic bytes
						offset = 1;
					};

					let parsed = parse_blk(&file.1[offset..], file_type.is_slim(), self.nm.clone())?;
					match format {
						BlkOutputFormat::Json(config) => {
							file.1 = parsed.as_ref_json(config)?.into_bytes();
						}
						BlkOutputFormat::BlkText => {
							file.1 = parsed.as_blk_text().into_bytes();
						}
					}
				}
				Ok(file.1)
			}

			// Default to the raw file
			_ => {
				Ok(file.1)
			}
		}
	}
}

fn maybe_blk(file: &File) -> bool {
	file.0.extension() == Some(OsStr::new("blk")) && file.1.len() > 0 && FileType::from_byte(file.1[0]).is_ok()
}