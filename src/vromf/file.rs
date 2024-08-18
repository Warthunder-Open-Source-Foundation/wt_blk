use std::fs;
use std::path::{Path, PathBuf};


#[derive(Debug, Clone)]
pub struct File {
	path: PathBuf,
	file: Vec<u8>,
}

impl File {
	pub fn new(p: impl Into<PathBuf>) -> color_eyre::Result<Self> {
		let path = p.into();
		Ok(Self {
			file: fs::read(&path)?,
			path,
		})
	}

	pub fn from_raw(path: PathBuf, file: Vec<u8>) -> Self {
		Self {
			path,
			file,
		}
	}

	pub fn split(self) -> (PathBuf, Vec<u8>) {
		(self.path, self.file)
	}


	pub fn path(&self) -> &Path {
		self.path.as_path()
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
}

