use std::{ffi::OsStr, fmt::{Debug, Formatter}, mem, path::{Path, PathBuf}, sync::Arc};
use std::io::{Cursor, Write};

use color_eyre::{eyre::ContextCompat, Help, Report};
use color_eyre::eyre::Context;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};
use zstd::dict::DecoderDictionary;

use crate::{
	blk::{
		blk_structure::BlkField,
		file::FileType,
		nm_file::NameMap,
		parser::parse_blk,
		zstd::decode_zstd,
	},
	vromf::{binary_container::decode_bin_vromf, inner_container::decode_inner_vromf},
};
use crate::vromf::header::Metadata;

/// Simple type alias for (Path, Data) pair
pub type File = (PathBuf, Vec<u8>);

#[derive()]
struct DictWrapper<'a>(DecoderDictionary<'a>);

impl Debug for DictWrapper<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "DecoderDictionary {{...}}")
	}
}


/// Unpacks vromf image into all internal files, optionally formatting binary BLK files
#[derive(Debug)]
pub struct VromfUnpacker<'a> {
	files: Vec<File>,
	dict: Option<Arc<DictWrapper<'a>>>,
	nm: Option<Arc<NameMap>>,
	metadata: Metadata,
}

/// Defines plaintext format should be exported to
#[derive(Copy, Clone, Debug)]
pub enum BlkOutputFormat {
	Json,
	BlkText,
}

#[derive(Copy, Clone, Debug)]
pub enum ZipFormat {
	Uncompressed,
	Compressed(u8),
}


impl VromfUnpacker<'_> {
	pub fn from_file(file: File) -> Result<Self, Report> {
		let (decoded, metadata) = decode_bin_vromf(&file.1)?;
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
			metadata,
		})
	}

	pub fn unpack_all(mut self, unpack_blk_into: Option<BlkOutputFormat>, apply_overrides: bool) -> Result<Vec<File>, Report> {
		// Important: We own self here, so "destroying" the files vector isn't an issue
		// meaning an option isnt required
		let files = mem::replace(&mut self.files, vec![]);
		files.into_par_iter()
			.map(|file| {
				self.unpack_file(file, unpack_blk_into, apply_overrides)
			})
			.collect::<Result<Vec<File>, Report>>()
	}

	pub fn unpack_all_to_zip(mut self, zip_format: ZipFormat, unpack_blk_into: Option<BlkOutputFormat>, apply_overrides: bool) -> Result<Vec<u8>, Report> {
		// Important: We own self here, so "destroying" the files vector isn't an issue
		// meaning an option isnt required
		let files = mem::replace(&mut self.files, Default::default());
		let unpacked = files.into_par_iter()
			.map(|file| {
				self.unpack_file(file, unpack_blk_into, apply_overrides)
			})
			.collect::<Result<Vec<File>, Report>>()?;

		let mut buf = Cursor::new(Vec::with_capacity(4096));
		let mut writer = ZipWriter::new(&mut buf);

		let (compression_level, compression_method) = match zip_format {
			ZipFormat::Uncompressed => {
				(0, CompressionMethod::STORE)
			}
			ZipFormat::Compressed(level) => {
				(level, CompressionMethod::DEFLATE)
			}
		};

		for (path, data) in unpacked.into_iter() {
			writer.start_file(path.to_string_lossy(),
							  FileOptions::default()
								  .compression_level(Some(compression_level as i32))
								  .compression_method(compression_method),
			)?;
			writer.write_all(&data)?;
		}

		let buf = mem::replace(writer.finish()?, Default::default());
		Ok(buf.into_inner())
	}

	pub fn unpack_one(
		&self,
		path_name: &Path,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
	) -> Result<File, Report> {
		let file = self
			.files
			.iter()
			.find(|e| e.0 == path_name)
			.context(format!("File {} was not found in VROMF", path_name.to_string_lossy()))
			.suggestion("Validate file-name and ensure it was typed correctly")?
			.to_owned();
		self.unpack_file(file, unpack_blk_into, apply_overrides)
	}

	pub fn unpack_file(&self, mut file: File, unpack_blk_into: Option<BlkOutputFormat>, apply_overrides: bool) -> Result<File, Report> {
		match () {
			_ if maybe_blk(&file) => {
				if let Some(format) = unpack_blk_into {
					let mut offset = 0;
					let file_type = FileType::from_byte(file.1[0])?;
					if file_type.is_zstd() {
						if file_type == FileType::FAT_ZSTD { offset += 1 }; // FAT_ZSTD has a leading byte indicating that its unpacked form is of the FAT format
						file.1 = decode_zstd(file_type, &file.1, self.dict.as_ref().map(|e| &e.0))?;
					} else {
						// uncompressed Slim and Fat files retain their initial magic bytes
						offset = 1;
					};

					let mut parsed =
						parse_blk(&file.1[offset..], file_type.is_slim(), self.nm.clone()).wrap_err(format!("{}", file.0.to_string_lossy()))?;
					match format {
						BlkOutputFormat::BlkText => {
							if apply_overrides {
								parsed.apply_overrides();
							}
							file.1 = parsed.as_blk_text()?.into_bytes();
						}
						BlkOutputFormat::Json => {
							file.1 = serde_json::to_string_pretty(&parsed.as_serde_obj(apply_overrides))?
								.into_bytes();
						}
					}
				}
				Ok(file)
			}

			// Default to the raw file
			_ => Ok(file),
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
					file.1 = decode_zstd(file_type, &file.1, self.dict.as_ref().map(|e| &e.0))?;
				} else {
					// uncompressed Slim and Fat files retain their initial magic bytes
					offset = 1;
				};

				Ok(parse_blk(
					&file.1[offset..],
					file_type.is_slim(),
					self.nm.clone(),
				)?)
			}
			_ => panic!("Not a blk"),
		}
	}
}

fn maybe_blk(file: &File) -> bool {
	file.0.extension() == Some(OsStr::new("blk"))
		&& file.1.len() > 0
		&& FileType::from_byte(file.1[0]).is_ok()
}