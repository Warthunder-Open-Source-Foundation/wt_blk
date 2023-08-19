use std::{
	ffi::OsStr,
	fmt::{Debug, Formatter},
	path::{Path, PathBuf},
	sync::Arc,
};

use color_eyre::{Help, Report};
use color_eyre::eyre::ContextCompat;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use zstd::dict::DecoderDictionary;

use crate::{
	blk::{
		blk_structure::BlkField,
		BlkOutputFormat,
		file::FileType,
		nm_file::NameMap,
		parser::parse_blk,
		zstd::decode_zstd,
	},
	vromf::{
		binary_container::decode_bin_vromf,
		inner_container::decode_inner_vromf,
	},
};

pub type File = (PathBuf, Vec<u8>);

#[derive()]
struct DictWrapper<'a>(DecoderDictionary<'a>);

impl Debug for DictWrapper<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "DecoderDictionary {{...}}")
	}
}

#[derive(Debug)]
pub struct VromfUnpacker<'a> {
	files: Vec<File>,
	dict: Option<Arc<DictWrapper<'a>>>,
	nm: Option<Arc<NameMap>>,
}

impl VromfUnpacker<'_> {
	pub fn from_file(file: File) -> Result<Self, Report> {
		let decoded = decode_bin_vromf(&file.1)?;
		let inner = decode_inner_vromf(&decoded)?;

		let nm = inner
			.iter()
			.find(|elem| elem.0.file_name() == Some(OsStr::new("nm")))
			.map(|elem| NameMap::from_encoded_file(&elem.1))
			.transpose()?
			.map(|elem| Arc::new(elem));

		let dict = inner
			.iter()
			.find(|elem| elem.0.extension() == Some(OsStr::new("dict")))
			.map(|elem| Arc::new(DictWrapper(DecoderDictionary::copy(&elem.1))));

		Ok(Self {
			files: inner,
			dict,
			nm,
		})
	}

	pub fn unpack_all(
		self,
		unpack_blk_into: Option<BlkOutputFormat>,
	) -> Result<Vec<File>, Report> {
		self.files
			.into_par_iter()
			.map(|mut file| {
				match () {
					_ if maybe_blk(&file) => {
						if let Some(format) = unpack_blk_into {
							let mut offset = 0;
							let file_type = FileType::from_byte(file.1[0])?;
							if file_type.is_zstd() {
								file.1 =
									decode_zstd(&file.1, self.dict.as_ref().map(|e| &e.0))?;
							} else {
								// uncompressed Slim and Fat files retain their initial magic bytes
								offset = 1;
							};

							let parsed =
								parse_blk(&file.1[offset..], file_type.is_slim(), self.nm.clone())?;
							match format {
								BlkOutputFormat::Json(config) => {
									file.1 = parsed.as_ref_json(config)?.into_bytes();
								}
								BlkOutputFormat::BlkText => {
									file.1 = parsed.as_blk_text().into_bytes();
								}
								BlkOutputFormat::JsonMerged => {
									file.1 = serde_json::to_string_pretty(&parsed.as_serde_obj())?.into_bytes();
								}
							}
						}
						Ok(file)
					}

					// Default to the raw file
					_ => Ok(file),
				}
			})
			.collect::<Result<Vec<File>, Report>>()
	}

	pub fn unpack_one(
		&self,
		path_name: &Path,
		unpack_blk_into: Option<BlkOutputFormat>,
	) -> Result<Vec<u8>, Report> {
		let mut file = self
			.files
			.iter()
			.find(|e| e.0 == path_name)
			.context("File {path_name} was not found in VROMF")
			.suggestion("Validate file-name and ensure it was typed correctly")?
			.to_owned();
		match () {
			_ if maybe_blk(&file) => {
				if let Some(format) = unpack_blk_into {
					let mut offset = 0;
					let file_type = FileType::from_byte(file.1[0])?;
					if file_type.is_zstd() {
						file.1 = decode_zstd(&file.1, self.dict.as_ref().map(|e| &e.0))?;
					} else {
						// uncompressed Slim and Fat files retain their initial magic bytes
						offset = 1;
					};

					let parsed =
						parse_blk(&file.1[offset..], file_type.is_slim(), self.nm.clone())?;
					match format {
						BlkOutputFormat::Json(config) => {
							file.1 = parsed.as_ref_json(config)?.into_bytes();
						}
						BlkOutputFormat::BlkText => {
							file.1 = parsed.as_blk_text().into_bytes();
						}
						BlkOutputFormat::JsonMerged => {
							file.1 = serde_json::to_string_pretty(&parsed.as_serde_obj())?.into_bytes();
						}
					}
				}
				Ok(file.1)
			}

			// Default to the raw file
			_ => Ok(file.1),
		}
	}
	// For debugging purposes
	pub fn unpack_one_to_field(&self, path_name: &Path) -> Result<BlkField, Report> {
		let mut file = self
			.files
			.iter()
			.find(|e| e.0 == path_name)
			.context("File {path_name} was not found in VROMF")
			.suggestion("Validate file-name and ensure it was typed correctly")?
			.to_owned();
		match () {
			_ if maybe_blk(&file) => {
				let mut offset = 0;
				let file_type = FileType::from_byte(file.1[0])?;
				if file_type.is_zstd() {
					file.1 = decode_zstd(&file.1, self.dict.as_ref().map(|e| &e.0))?;
				} else {
					// uncompressed Slim and Fat files retain their initial magic bytes
					offset = 1;
				};

				Ok(parse_blk(&file.1[offset..], file_type.is_slim(), self.nm.clone())?)
			}
			_ => panic!("Not a blk")
		}
	}
}

fn maybe_blk(file: &File) -> bool {
	file.0.extension() == Some(OsStr::new("blk"))
		&& file.1.len() > 0
		&& FileType::from_byte(file.1[0]).is_ok()
}
