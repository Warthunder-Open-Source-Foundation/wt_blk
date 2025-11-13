use std::{
	ffi::OsStr,
	fmt::{Debug, Formatter},
	io::{Cursor, Write},
	mem,
	ops::{Deref, Not},
	path::{Path, PathBuf},
	str::FromStr,
	sync::Arc,
};

use color_eyre::{
	Help, Report,
	eyre::{Context, ContextCompat, eyre},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use regex::Regex;
use wt_version::Version;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};
use zstd::dict::DecoderDictionary;

use crate::blk::blk_type::BlkFormatting;
use crate::{
	blk,
	blk::{name_map::NameMap, util::maybe_blk},
	vromf::{
		File, binary_container::decode_bin_vromf, header::Metadata,
		inner_container::decode_inner_vromf,
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
pub struct VromfUnpacker {
	files: Vec<File>,
	dict: Option<Arc<DictWrapper>>,
	nm: Option<Arc<NameMap>>,
	metadata: Metadata,
}

/// Defines plaintext format should be exported to
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum BlkOutputFormat {
	Json,
	BlkText,
	BlkCompact,
}

impl BlkOutputFormat {
	pub fn map_to_formatter(self) -> BlkFormatting {
		match self {
			BlkOutputFormat::Json | BlkOutputFormat::BlkText => BlkFormatting::standard(),
			BlkOutputFormat::BlkCompact => BlkFormatting::compact(),
		}
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ZipFormat {
	Uncompressed,
	Compressed(u8),
}

#[derive(Debug, Clone)]
pub enum FileFilter {
	All,
	OneFolder {
		remove_base: bool,
		prefix: Arc<PathBuf>,
	},
	FullPathRegex {
		rex: Arc<Regex>,
	},
}

impl FileFilter {
	pub fn accept(&self, file: &File) -> bool {
		match self {
			FileFilter::All => true,
			FileFilter::OneFolder { prefix, .. } => file.path().starts_with(prefix.as_ref()),
			FileFilter::FullPathRegex { rex } => rex.is_match(&file.path().to_string_lossy()),
		}
	}

	pub fn base_path_start(&self) -> usize {
		match self {
			FileFilter::OneFolder { prefix, .. } => prefix.to_string_lossy().len(),
			_ => 0,
		}
	}

	pub fn from_regexstr(re: &str) -> Result<Self, Report> {
		Ok(Self::FullPathRegex {
			rex: Arc::new(Regex::new(re).context(format!("Invalid regex: {}", re))?),
		})
	}

	pub const fn all() -> Self {
		Self::All
	}

	pub fn one_folder(prefix: Arc<PathBuf>, remove_base: bool) -> Self {
		Self::OneFolder {
			remove_base,
			prefix,
		}
	}
}

impl VromfUnpacker {
	// TODO: dump_parsed_nm should maybe be an argument passed to the other unpack functions, not the struct
	pub fn from_file(file: &File, validate: bool, dump_parsed_nm: bool) -> Result<Self, Report> {
		let (decoded, metadata) = decode_bin_vromf(file.buf(), validate)?;
		let mut inner = decode_inner_vromf(&decoded, validate)?;

		let nm = inner
			.iter()
			.find(|elem| elem.path().file_name() == Some(OsStr::new("nm")))
			.map(|elem| NameMap::from_encoded_file(&elem.buf()))
			.transpose()?
			.map(|elem| Arc::new(elem));

		if let Some(nm) = nm.as_ref()
			&& dump_parsed_nm
		{
			let mut sorted = (*nm.parsed).clone();
			sorted.sort_by(|a, b| a.cmp(b));

			inner.push(File::from_raw(
				PathBuf::from_str("nm.txt")?,
				sorted
					.iter()
					.map(|s| s.as_str())
					.collect::<Vec<_>>()
					.join("\n")
					.into_bytes(),
			));
		}

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
			.map(|file| self.unpack_file(file, unpack_blk_into, apply_overrides, FileFilter::All))
			.collect::<Result<Vec<File>, Report>>()
	}

	/// Skips the buffering step and directly writes the file to disk, using a provided writer
	pub fn unpack_all_with_writer<W: Write>(
		mut self,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
		writer: impl FnOnce(&mut File) -> Result<W, Report> + Sync + Send + Copy,
		// Runs unpacking in the global rayon threadpool if true, otherwise its single threaded
		// false increases global throughput when executed from a threadpool,
		// but slower when individual calls are performed
		threaded: bool,
	) -> Result<(), Report> {
		// Important: We own self here, so "destroying" the files vector isn't an issue
		// Due to partial moving rules this is necessary
		let files = mem::replace(&mut self.files, vec![]);

		// TODO: Figure out some way to deduplicate this
		// ParIter and Iter are obv. incompatible so this might need macro magic of sorts
		if threaded {
			files
				.into_par_iter()
				.panic_fuse()
				.map(|mut file| {
					let mut w = writer(&mut file)?;
					self.unpack_file_with_writer(
						&mut file,
						unpack_blk_into,
						apply_overrides,
						&mut w,
						FileFilter::All,
					)?;
					Ok(())
				})
				.collect::<Result<(), Report>>()
		} else {
			files
				.into_iter()
				.map(|mut file| {
					let mut w = writer(&mut file)?;
					self.unpack_file_with_writer(
						&mut file,
						unpack_blk_into,
						apply_overrides,
						&mut w,
						FileFilter::All,
					)?;
					Ok(())
				})
				.collect::<Result<(), Report>>()
		}
	}

	pub fn unpack_subfolder_to_zip(
		&self,
		zip_format: ZipFormat,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
		// Runs unpacking in the global rayon threadpool if true, otherwise its single threaded
		// false increases global throughput when executed from a threadpool,
		// but slower when individual calls are performed
		threaded: bool,
		filter: FileFilter,
	) -> Result<Vec<u8>, Report> {
		// TODO: Figure out some way to deduplicate this
		// ParIter and Iter are obv. incompatible so this might need macro magic of sorts
		let files = &self.files;
		let unpacked = if threaded {
			files
				.into_par_iter()
				.panic_fuse()
				.cloned()
				.map(|file| {
					self.unpack_file(file, unpack_blk_into, apply_overrides, filter.clone())
				})
				.collect::<Result<Vec<File>, Report>>()?
		} else {
			files
				.iter()
				.cloned()
				.map(|file| {
					self.unpack_file(file, unpack_blk_into, apply_overrides, filter.clone())
				})
				.collect::<Result<Vec<File>, Report>>()?
		};

		let mut buf = Cursor::new(Vec::with_capacity(4096));
		let mut writer = ZipWriter::new(&mut buf);

		let (compression_level, compression_method) = match zip_format {
			ZipFormat::Uncompressed => (None, CompressionMethod::STORE),
			ZipFormat::Compressed(level) => (Some(level as i64), CompressionMethod::DEFLATE),
		};

		for f in unpacked.into_iter() {
			writer.start_file(
				&f.path().to_string_lossy()[filter.base_path_start()..],
				SimpleFileOptions::default()
					.compression_level(compression_level)
					.compression_method(compression_method),
			)?;
			writer.write_all(f.buf())?;
		}

		let buf = mem::replace(writer.finish()?, Default::default());
		Ok(buf.into_inner())
	}

	pub fn unpack_all_to_zip(
		&self,
		zip_format: ZipFormat,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
		// Runs unpacking in the global rayon threadpool if true, otherwise its single threaded
		threaded: bool,
	) -> Result<Vec<u8>, Report> {
		self.unpack_subfolder_to_zip(
			zip_format,
			unpack_blk_into,
			apply_overrides,
			threaded,
			FileFilter::All,
		)
	}

	pub fn unpack_one(
		&self,
		path_name: &Path,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
		file_filter: FileFilter,
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
		self.unpack_file(file, unpack_blk_into, apply_overrides, file_filter)
	}

	pub fn unpack_file(
		&self,
		mut file: File,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
		filter: FileFilter,
	) -> Result<File, Report> {
		let mut buf = Cursor::new(Vec::with_capacity(4096));
		self.unpack_file_with_writer(
			&mut file,
			unpack_blk_into,
			apply_overrides,
			&mut buf,
			filter,
		)?;
		*file.buf_mut() = buf.into_inner();
		Ok(file)
	}

	/// This is where the unpacking actually occurs
	pub fn unpack_file_with_writer(
		&self,
		file: &mut File,
		unpack_blk_into: Option<BlkOutputFormat>,
		apply_overrides: bool,
		mut writer: impl Write,
		filter: FileFilter,
	) -> Result<(), Report> {
		if filter.accept(file).not() {
			return Ok(());
		}
		match () {
			_ if maybe_blk(&file) => {
				if let Some(format) = unpack_blk_into {
					let mut parsed = blk::unpack_blk(file.buf_mut(), self.dict(), self.nm.clone())
						.context(format!("unpacking {}", file.path().to_string_lossy()))?;

					match format {
						BlkOutputFormat::BlkText | BlkOutputFormat::BlkCompact => {
							if apply_overrides {
								parsed.apply_overrides(false);
							}
							writer.write_all(
								parsed.as_blk_text(format.map_to_formatter())?.as_bytes(),
							)?;
						},
						BlkOutputFormat::Json => {
							parsed.merge_fields()?;
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
			.unpack_one(Path::new("version"), None, false, FileFilter::All)
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
