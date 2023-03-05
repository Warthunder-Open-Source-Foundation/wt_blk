


#[cfg(test)]
mod test {
	use crate::util::time;
	use std::thread::sleep;
	use std::time::Duration;

	#[test]
	fn test_any() {
		time!(sleep(Duration::from_millis(100)));
	}

	#[test]
	fn test_str() {
		let out = time!(String::from("yeet"));
		assert_eq!("yeet", out);
		dbg!();
	}
}