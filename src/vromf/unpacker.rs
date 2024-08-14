use std::{
	ffi::OsStr,
	fmt::{Debug, Formatter},
	io::{Cursor, Write},
	mem,
	ops::Deref,
	path::{Path, PathBuf},
	str::FromStr,
	sync::Arc,
};

use color_eyre::{
	eyre::{eyre, ContextCompat},
	Help,
	Report,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use wt_version::Version;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};
use zstd::dict::DecoderDictionary;

use crate::{
	blk,
	blk::{nm_file::NameMap, util::maybe_blk},
	vromf::{
		binary_container::decode_bin_vromf,
		header::Metadata,
		inner_container::decode_inner_vromf,
	},
};

/// Simple type alias for (Path, Data) pair
pub type File = (PathBuf, Vec<u8>);

#[derive()]
struct DictWrapper<'a>(DecoderDictionary<'a>);

impl Debug for DictWrapper<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "DecoderDictionary {{...}}")
	}
}

impl<'a> Deref for DictWrapper<'a> {
	type Target = DecoderDictionary<'a>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Unpacks vromf image into all internal files, optionally formatting binary BLK files
#[derive(Debug)]
pub struct VromfUnpacker<'a> {
	files:    Vec<File>,
	dict:     Option<Arc<DictWrapper<'a>>>,
	nm:       Option<Arc<NameMap>>,
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
	pub fn from_file(file: File, validate: bool) -> Result<Self, Report> {
		let (decoded, metadata) = decode_bin_vromf(&file.1, validate)?;
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

	pub fn unpack_all(
		mut self,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
	) -> Result<Vec<File>, Report> {
		// Important: We own self here, so "destroying" the files vector isn't an issue
		// Due to partial moving rules this is necessary
		let files = mem::replace(&mut self.files, vec![]);
		files
			.into_par_iter()
			.panic_fuse()
			.map(|file| self.unpack_file(file, unpack_blk_into, apply_overrides))
			.collect::<Result<Vec<File>, Report>>()
	}

	/// Skips the buffering step and directly writes the file to disk, using a provided writer
	pub fn unpack_all_with_writer<W: Write>(
		mut self,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
		writer: impl FnOnce(&mut File) -> Result<W, Report> + Sync + Send + Copy,
	) -> Result<(), Report> {
		// Important: We own self here, so "destroying" the files vector isn't an issue
		// Due to partial moving rules this is necessary
		let files = mem::replace(&mut self.files, vec![]);
		files
			.into_par_iter()
			.panic_fuse()
			.map(|mut file| {
				let mut w = writer(&mut file)?;
				self.unpack_file_with_writer(file, unpack_blk_into, apply_overrides, &mut w)?;
				Ok(())
			})
			.collect::<Result<(), Report>>()
	}

	pub fn unpack_all_to_zip(
		mut self,
		zip_format: ZipFormat,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
	) -> Result<Vec<u8>, Report> {
		// Important: We own self here, so "destroying" the files vector isn't an issue
		// Due to partial moving rules this is necessary
		let files = mem::replace(&mut self.files, Default::default());
		let unpacked = files
			.into_par_iter()
			.panic_fuse()
			.map(|file| self.unpack_file(file, unpack_blk_into, apply_overrides))
			.collect::<Result<Vec<File>, Report>>()?;

		let mut buf = Cursor::new(Vec::with_capacity(4096));
		let mut writer = ZipWriter::new(&mut buf);

		let (compression_level, compression_method) = match zip_format {
			ZipFormat::Uncompressed => (0, CompressionMethod::STORE),
			ZipFormat::Compressed(level) => (level, CompressionMethod::DEFLATE),
		};

		for (path, data) in unpacked.into_iter() {
			writer.start_file(
				path.to_string_lossy(),
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
			.context(format!(
				"File {} was not found in VROMF",
				path_name.to_string_lossy()
			))
			.suggestion("Validate file-name and ensure it was typed correctly")?
			.to_owned();
		self.unpack_file(file, unpack_blk_into, apply_overrides)
	}

	pub fn unpack_file(
		&self,
		mut file: File,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
	) -> Result<File, Report> {
		match () {
			_ if maybe_blk(&file) => {
				if let Some(format) = unpack_blk_into {
					let mut parsed = blk::unpack_blk(&mut file.1, self.dict(), self.nm.clone())?;
					if apply_overrides {
						parsed.apply_overrides();
					}

					match format {
						BlkOutputFormat::BlkText => {
							file.1 = parsed.as_blk_text()?.into_bytes();
						},
						BlkOutputFormat::Json => {
							parsed.merge_fields();
							parsed.as_serde_json_streaming(&mut file.1)?;
						},
					}
				}
				Ok(file)
			},

			// Default to the raw file
			_ => Ok(file),
		}
	}

	pub fn unpack_file_with_writer(
		&self,
		mut file: File,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
		mut writer: impl Write,
	) -> Result<(), Report> {
		match () {
			_ if maybe_blk(&file) => {
				if let Some(format) = unpack_blk_into {
					let mut parsed = blk::unpack_blk(&mut file.1, self.dict(), self.nm.clone())?;

					match format {
						BlkOutputFormat::BlkText => {
							if apply_overrides {
								parsed.apply_overrides();
							}
							writer.write_all(parsed.as_blk_text()?.as_bytes())?;
						},
						BlkOutputFormat::Json => {
							parsed.merge_fields();
							if apply_overrides {
								parsed.apply_overrides();
							}
							parsed.as_serde_json_streaming(&mut writer)?;
						},
					}
				}
			},
			// Default to the raw file
			_ => {
				writer.write_all(&file.1)?;
			},
		}
		writer.flush()?;
		Ok(())
	}

	pub fn query_versions(&self) -> Result<Vec<Version>, Report> {
		let mut versions = vec![];
		if let Some(meta) = self.metadata.version {
			versions.push(meta);
		};

		if let Ok((_, version_file)) = self.unpack_one(Path::new("version"), None, false) {
			let s = String::from_utf8(version_file)?;
			versions.push(
				Version::from_str(&s).map_err(|_| eyre!("Invalid version file contents: {s}"))?,
			);
		}

		Ok(versions)
	}

	pub fn latest_version(&self) -> Result<Option<Version>, Report> {
		let mut versions = self.query_versions()?;
		versions.sort_unstable();
		Ok(versions.last().map(|e| e.to_owned()))
	}

	pub fn list_files(&self) {
		for (path, _) in &self.files {
			println!("{}", path.to_string_lossy());
		}
	}

	pub fn dict(&self) -> Option<&DecoderDictionary<'_>> {
		self.dict.as_deref().map(Deref::deref)
	}
}
