macro_rules! time {
    ($e:expr) => {{
        let start = std::time::Instant::now();
        let output = $e;
        println!("{}:{} {:?}", file!{}, line!{},start.elapsed());
        drop(start);
        output
    }};
}

pub(crate) use time;

pub(crate) fn debug_hex(hex: &[u8]) {
    eprintln!("{:?}", hex.iter().map( |x|format!("0x{x:X}")).collect::<Vec<_>>());
}
