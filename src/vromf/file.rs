use std::{
	fs,
	fs::Metadata,
	io::Read,
	path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct File {
	path: PathBuf,
	file: Vec<u8>,
	// Present when read from disk, not present when created from VROMF
	meta: Option<Metadata>,
}

impl File {
	pub fn new(p: impl Into<PathBuf>) -> color_eyre::Result<Self> {
		let path = p.into();
		let mut f = fs::File::open(&path)?;
		let mut buf = Vec::with_capacity(1024);
		f.read_to_end(&mut buf)?;
		Ok(Self {
			file: buf,
			path,
			meta: Some(f.metadata()?),
		})
	}

	pub fn from_raw_with_meta(path: PathBuf, file: Vec<u8>, meta: Metadata) -> Self {
		Self {
			path,
			file,
			meta: Some(meta),
		}
	}

	pub fn from_raw(path: PathBuf, file: Vec<u8>) -> Self {
		Self {
			path,
			file,
			meta: None,
		}
	}

	pub fn split(self) -> (PathBuf, Vec<u8>) {
		(self.path, self.file)
	}

	pub fn path(&self) -> &Path {
		self.path.as_path()
	}

	pub fn path_mut(&mut self) -> &mut PathBuf {
		&mut self.path
	}

	pub fn buf(&self) -> &[u8] {
		self.file.as_slice()
	}

	pub fn buf_mut(&mut self) -> &mut Vec<u8> {
		&mut self.file
	}

	pub fn as_ref(&self) -> (&Path, &[u8]) {
		(self.path(), self.buf())
	}

	pub fn meta(&self) -> &Option<Metadata> {
		&self.meta
	}
}
