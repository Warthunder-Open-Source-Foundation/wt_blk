pub fn bytes_to_offset(input: &[u8]) -> Option<usize> {
	if input.len() != 4 {
		return None;
	}

	Some(u32::from_le_bytes([
		input[0],
		input[1],
		input[2],
		input[3],
	]) as usize)
}

pub fn bytes_to_float(input: &[u8]) -> Option<f32> {
	if input.len() != 4 {
		return None;
	}

	Some(f32::from_le_bytes([
		input[0],
		input[1],
		input[2],
		input[3],
	]))
}

pub fn bytes_to_int(input: &[u8]) -> Option<u32> {
	if input.len() != 4 {
		return None;
	}

	Some(u32::from_le_bytes([
		input[0],
		input[1],
		input[2],
		input[3],
	]))
}

pub fn bytes_to_long(input: &[u8]) -> Option<u64> {
	if input.len() != 8 {
		return None;
	}

	Some(u64::from_le_bytes([
		input[0],
		input[1],
		input[2],
		input[3],
		input[4],
		input[5],
		input[6],
		input[7],
	]))
}