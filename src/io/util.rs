use std::{fs, fs::ReadDir, path::PathBuf};

#[allow(unused)]
pub fn test_parse_dir(dir: ReadDir, stack: &mut Vec<(PathBuf, Vec<u8>)>) {
	for file in dir {
		let f = file.unwrap();
		if f.file_type().unwrap().is_dir() {
			test_parse_dir(f.path().read_dir().unwrap(), stack);
		} else {
			stack.push((f.path(), fs::read(f.path()).unwrap()));
		}
	}
}
