use std::ffi::{CStr, CString};
use std::process::{abort, exit};

#[derive(Debug, PartialOrd, PartialEq)]
pub enum BlkType {
	Str(String),
	Int(u32),
	Int2([u32; 2]),
	Int3([u32; 3]),
	Long(u64),
	Float(f32),
	Float2([f32; 2]),
	Float3([f32; 3]),
	Float4([f32; 4]),
	Float12([f32; 12]),
	Bool(bool),
	/// Stored as RGBA
	Color([u8; 4]),
}

impl BlkType {
	/// Type ID as corresponding to its type_code
	/// Field is a 4 byte long region that either contains the final value or offset for the data region
	/// data_region is for non-32 bit data
	pub fn from_raw_param_info(type_id: u8, field: &[u8], data_region: &[u8]) -> Option<Self> {

		// Make sure the field is properly sized
		if field.len() != 4 {
			return None;
		}

		match type_id {
			0x06 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 16)];
				return Some(Self::Float4([
					bytes_to_float(&data_region[0..4])?,
					bytes_to_float(&data_region[4..8])?,
					bytes_to_float(&data_region[8..12])?,
					bytes_to_float(&data_region[12..16])?,
				]));
			}
			0x04 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 8)];
				return Some(Self::Float2([
					bytes_to_float(&data_region[0..4])?,
					bytes_to_float(&data_region[4..8])?,
				]));
			}
			0x02 => {
				return Some(Self::Int(bytes_to_int(field)?));
			}
			0x0c => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 8)];
				return Some(Self::Long(bytes_to_long(data_region)?));
			}
			0x01 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..];
				let cstr = CStr::from_bytes_until_nul(data_region).unwrap();
				let rstr = cstr.to_str().ok()?.to_owned();
				return Some(Self::Str(rstr));
			}
			0x09 => {
				return Some(Self::Bool(field[0] != 0))
			}
			0x0a => {
				// Game stores them in BGRA order
				return Some(Self::Color([
					field[2],
					field[1],
					field[0],
					field[3],
				]))
			}
			0x07 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 8)];
				return Some(Self::Int2([
					bytes_to_int(&data_region[0..4])?,
					bytes_to_int(&data_region[4..8])?,
				]));
			}
			0x03 => {
				return Some(Self::Float(bytes_to_float(field)?))
			}
			0x08 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 12)];
				return Some(Self::Int3([
					bytes_to_int(&data_region[0..4])?,
					bytes_to_int(&data_region[4..8])?,
					bytes_to_int(&data_region[8..12])?,
				]));
			}
			0x05 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 12)];
				return Some(Self::Float3([
					bytes_to_float(&data_region[0..4])?,
					bytes_to_float(&data_region[4..8])?,
					bytes_to_float(&data_region[8..12])?,
				]));
			}
			0x0b => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 48)];
				return Some(Self::Float12([
					bytes_to_float(&data_region[0..4])?,
					bytes_to_float(&data_region[4..8])?,
					bytes_to_float(&data_region[8..12])?,
					bytes_to_float(&data_region[12..16])?,
					bytes_to_float(&data_region[16..20])?,
					bytes_to_float(&data_region[20..24])?,
					bytes_to_float(&data_region[24..28])?,
					bytes_to_float(&data_region[28..32])?,
					bytes_to_float(&data_region[32..36])?,
					bytes_to_float(&data_region[36..40])?,
					bytes_to_float(&data_region[40..44])?,
					bytes_to_float(&data_region[44..48])?,
				]));
			}
			_ => { return None; }
		}
	}

	pub const fn type_code(&self) -> u8 {
		match self {
			BlkType::Str(_) => { 0x01 }
			BlkType::Int(_) => { 0x02 }
			BlkType::Int2(_) => { 0x7 }
			BlkType::Int3(_) => { 0x08 }
			BlkType::Long(_) => { 0x0c }
			BlkType::Float(_) => { 0x04 }
			BlkType::Float2(_) => { 0x04 }
			BlkType::Float3(_) => { 0x05 }
			BlkType::Float4(_) => { 0x06 }
			BlkType::Float12(_) => { 0x0b }
			BlkType::Bool(_) => { 0x09 }
			BlkType::Color(_) => { 0x0a }
		}
	}
	pub const fn is_inline(&self) -> bool {
		match self {
			BlkType::Str(_) => { false }
			BlkType::Int(_) => { true }
			BlkType::Int2(_) => { false }
			BlkType::Int3(_) => { false }
			BlkType::Long(_) => { false }
			BlkType::Float(_) => { true }
			BlkType::Float2(_) => { false }
			BlkType::Float3(_) => { false }
			BlkType::Float4(_) => { false }
			BlkType::Float12(_) => { false }
			BlkType::Bool(_) => { true }
			BlkType::Color(_) => { true }
		}
	}
	pub fn size_bytes(&self) -> usize {
		match self {
			BlkType::Str(inner) => { inner.len() }
			BlkType::Int(_) => { 4 }
			BlkType::Int2(_) => { 8 }
			BlkType::Int3(_) => { 12 }
			BlkType::Long(_) => { 8 }
			BlkType::Float(_) => { 4 }
			BlkType::Float2(_) => { 8 }
			BlkType::Float3(_) => { 12 }
			BlkType::Float4(_) => { 16 }
			BlkType::Float12(_) => { 48 }
			BlkType::Bool(_) => { 4 }
			BlkType::Color(_) => { 4 }
		}
	}
}

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