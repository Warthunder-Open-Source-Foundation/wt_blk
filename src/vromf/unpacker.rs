use std::{
	ffi::OsStr,
	fmt::{Debug, Formatter},
	io::{Cursor, Write},
	mem,
	ops::Deref,
	path::Path,
	str::FromStr,
	sync::Arc,
};
use std::ops::RangeFrom;
use color_eyre::{
	eyre::{eyre, ContextCompat},
	Help,
	Report,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use wt_version::Version;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};
use zstd::dict::DecoderDictionary;

use crate::{
	blk,
	blk::{nm_file::NameMap, util::maybe_blk},
	vromf::{
		binary_container::decode_bin_vromf,
		header::Metadata,
		inner_container::decode_inner_vromf,
		File,
	},
};

// TODO: Check if this leaks, or if the FFI drops the contents appropriately
// TODO: Implement https://docs.rs/zstd/latest/zstd/dict/struct.DecoderDictionary.html#method.new once it is no longer experimental
#[derive()]
struct DictWrapper(DecoderDictionary<'static>);

impl Debug for DictWrapper {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "DecoderDictionary {{...}}")
	}
}

impl<'a> Deref for DictWrapper {
	type Target = DecoderDictionary<'static>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Unpacks vromf image into all internal files, optionally formatting binary BLK files
#[derive(Debug, Clone)]
pub struct VromfUnpacker{
	files:    Vec<File>,
	dict:     Option<Arc<DictWrapper>>,
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

impl VromfUnpacker {
	pub fn from_file(file: &File, validate: bool) -> Result<Self, Report> {
		let (decoded, metadata) = decode_bin_vromf(file.buf(), validate)?;
		let inner = decode_inner_vromf(&decoded, validate)?;

		let nm = inner
			.iter()
			.find(|elem| elem.path().file_name() == Some(OsStr::new("nm")))
			.map(|elem| NameMap::from_encoded_file(&elem.buf()))
			.transpose()?
			.map(|elem| Arc::new(elem));

		let dict = inner
			.iter()
			.find(|elem| elem.path().extension() == Some(OsStr::new("dict")))
			.map(|elem| Arc::new(DictWrapper(DecoderDictionary::copy(&elem.buf()))));

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
				self.unpack_file_with_writer(&mut file, unpack_blk_into, apply_overrides, &mut w)?;
				Ok(())
			})
			.collect::<Result<(), Report>>()
	}

	pub fn unpack_subfolder_to_zip(
		mut self,
		subfolder: &str,
		// removes subfolder from start of path
		// replacing for example `/gamedata/foo/bar` to `/foo/bar` when requesting `/gamedata`
		remove_root: bool,
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
			.filter(|f|f.path().starts_with(subfolder))
			.map(|file| self.unpack_file(file, unpack_blk_into, apply_overrides))
			.collect::<Result<Vec<File>, Report>>()?;

		let mut buf = Cursor::new(Vec::with_capacity(4096));
		let mut writer = ZipWriter::new(&mut buf);

		let (compression_level, compression_method) = match zip_format {
			ZipFormat::Uncompressed => (0, CompressionMethod::STORE),
			ZipFormat::Compressed(level) => (level, CompressionMethod::DEFLATE),
		};

		for f in unpacked.into_iter() {
			writer.start_file(
				&f.path().to_string_lossy()[if remove_root { subfolder.len().. } else { 0.. }],
				SimpleFileOptions::default()
					.compression_level(Some(compression_level as _))
					.compression_method(compression_method),
			)?;
			writer.write_all(f.buf())?;
		}

		let buf = mem::replace(writer.finish()?, Default::default());
		Ok(buf.into_inner())
	}

	pub fn unpack_all_to_zip(
		self,
		zip_format: ZipFormat,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
	) -> Result<Vec<u8>, Report> {
		self.unpack_subfolder_to_zip("", false, zip_format, unpack_blk_into, apply_overrides)
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
			.find(|e| e.path() == path_name)
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
		let mut buf = Cursor::new(Vec::with_capacity(4096));
		self.unpack_file_with_writer(&mut file, unpack_blk_into, apply_overrides, &mut buf)?;
		*file.buf_mut() = buf.into_inner();
		Ok(file)
	}

	pub fn unpack_file_with_writer(
		&self,
		file: &mut File,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
		mut writer: impl Write,
	) -> Result<(), Report> {
		match () {
			_ if maybe_blk(&file) => {
				if let Some(format) = unpack_blk_into {
					let mut parsed = blk::unpack_blk(file.buf_mut(), self.dict(), self.nm.clone())?;

					match format {
						BlkOutputFormat::BlkText => {
							if apply_overrides {
								parsed.apply_overrides(false);
							}
							writer.write_all(parsed.as_blk_text()?.as_bytes())?;
						},
						BlkOutputFormat::Json => {
							parsed.merge_fields();
							if apply_overrides {
								parsed.apply_overrides(true);
							}
							parsed.as_serde_json_streaming(&mut writer)?;
						},
					}
				} else {
					// Default to the raw file
					writer.write_all(file.buf())?;
				}
			},
			// Default to the raw file
			_ => {
				writer.write_all(file.buf())?;
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

		if let Ok((_, version_file)) = self
			.unpack_one(Path::new("version"), None, false)
			.map(|e| e.split())
		{
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
		for f in &self.files {
			println!("{}", f.path().to_string_lossy());
		}
	}

	pub fn dict(&self) -> Option<&DecoderDictionary<'_>> {
		self.dict.as_deref().map(Deref::deref)
	}
	pub fn nm(&self) -> Option<Arc<NameMap>> {
		self.nm.clone()
	}
}
