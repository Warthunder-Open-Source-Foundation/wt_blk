use crate::vromf::enums::Packing;

pub fn pack_type_from_aligned(input: u32) -> Option<(Packing, u32)> {
	const SIZE_MASK: u32 = 0b0000001111111111111111111111111;

	// Yields the first 6 bytes
	let pack_type_raw_aligned = (input.to_be_bytes()[0]) >> 2;
	let pack_type = Packing::try_from(pack_type_raw_aligned).ok()?;

	// yields the last 26 bytes
	let pack_size = input & SIZE_MASK;
	Some((pack_type, pack_size))
}