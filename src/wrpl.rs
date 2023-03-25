

#[cfg(test)]
mod test {
	use std::fs;

	#[test]
	fn extract_wrpl_1_floats()  {
		let file = fs::read("./samples/decompressed wrplu_1").unwrap();

		let mut floats = vec![];
		let float_mask = &[0x11, 0x00, 0x01, 0x60];
		for (i, window) in file.windows(4).enumerate() {
			if float_mask == window {
				let all_floats = &file[(i + 4)..((i + 4) + 3 * 4)];
				let parsed_floats = all_floats.chunks(4).map(|bin|f32::from_le_bytes([bin[0],bin[1],bin[2],bin[3],])).map(|x|format!("{x:<15}")).collect::<Vec<_>>();
				floats.push(parsed_floats.join("\t"));
			}
		}
		fs::write("floats.txt", floats.join("\n")).unwrap();
	}
}