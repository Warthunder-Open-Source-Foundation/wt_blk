use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::fs::ReadDir;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::PathBuf;
use std::time::Instant;

#[test]
fn read_seq() {
    let mut out = vec![];
    let start = Instant::now();
    test_parse_dir(
        fs::read_dir("./samples/vromfs/aces.vromfs.bin_u").unwrap(),
        &mut out,
    );
    // println!("Reading: {:?}", start.elapsed());

    let start = Instant::now();
    for file in out {
        fs::write(
            format!(
                "{}{}",
                "./samples/vromfs/out/",
                file.0.file_name().unwrap().to_str().unwrap()
            ),
            &file.1,
        )
        .unwrap();
    }
    // println!("Writing: {:?}", start.elapsed());
}

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
