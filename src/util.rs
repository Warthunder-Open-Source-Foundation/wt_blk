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
