use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use crate::binary::nm_file::NameMap;

pub type BlkCow<'a> = Cow<'a, str>;

#[derive(Debug, PartialOrd, PartialEq, Clone, Serialize, Deserialize)]
pub enum BlkType<'a> {
	Str(BlkCow<'a>),
	Int(u32),
	Int2([u32; 2]),
	Int3([u32; 3]),
	Long(u64),
	Float(f32),
	Float2([f32; 2]),
	Float3([f32; 3]),
	Float4([f32; 4]),
	Float12(Box<[f32; 12]>),
	Bool(bool),
	/// Stored as RGBA
	Color([u8; 4]),
}

impl <'a> BlkType<'a> {
	/// Type ID as corresponding to its type_code
	/// Field is a 4 byte long region that either contains the final value or offset for the data region
	/// data_region is for non-32 bit data
	pub fn from_raw_param_info(type_id: u8, field: &'a [u8], data_region: &'a [u8], name_map: Rc<Vec<BlkCow<'a>>>) -> Option<Self> {

		// Make sure the field is properly sized
		if field.len() != 4 {
			return None;
		}

		return match type_id {
			0x01 => {
				// Explanation:
				// Strings have their offset encoded as a LE integer constructed from 31 bits
				// The first bit in their field is an indicator whether or not to search in the regular data region or name map
				// The remaining bytes represent the integer
				let offset = u32::from_le_bytes([field[0], field[1], field[2], field[3]]); // Construct int from the individual bytes
				let in_nm = (offset >> 31) == 1; // Compare first bit to check where to look
				let offset = i32::MAX as u32 & offset; // Set first byte to 0
				let res: BlkCow =  if in_nm {
					name_map[offset as usize].clone()
				} else {
					let data_region = &data_region[(offset as usize)..];
					NameMap::parse_name_section(data_region).remove(0)
				};

				Some(Self::Str(res))
			}
			0x02 => {
				Some(Self::Int(bytes_to_int(field)?))
			}
			0x03 => {
				Some(Self::Float(bytes_to_float(field)?))
			}
			0x04 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 8)];
				Some(Self::Float2([
					bytes_to_float(&data_region[0..4])?,
					bytes_to_float(&data_region[4..8])?,
				]))
			}
			0x05 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 12)];
				Some(Self::Float3([
					bytes_to_float(&data_region[0..4])?,
					bytes_to_float(&data_region[4..8])?,
					bytes_to_float(&data_region[8..12])?,
				]))
			}
			0x06 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 16)];
				Some(Self::Float4([
					bytes_to_float(&data_region[0..4])?,
					bytes_to_float(&data_region[4..8])?,
					bytes_to_float(&data_region[8..12])?,
					bytes_to_float(&data_region[12..16])?,
				]))
			}
			0x07 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 8)];
				Some(Self::Int2([
					bytes_to_int(&data_region[0..4])?,
					bytes_to_int(&data_region[4..8])?,
				]))
			}
			0x08 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 12)];
				Some(Self::Int3([
					bytes_to_int(&data_region[0..4])?,
					bytes_to_int(&data_region[4..8])?,
					bytes_to_int(&data_region[8..12])?,
				]))
			}
			0x09 => {
				Some(Self::Bool(field[0] != 0))
			}
			0x0a => {
				// Game stores them in BGRA order
				Some(Self::Color([
					field[2],
					field[1],
					field[0],
					field[3],
				]))
			}
			0x0b => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 48)];
				Some(Self::Float12(Box::new([
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
				])))
			}
			0x0c => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 8)];
				Some(Self::Long(bytes_to_long(data_region)?))
			}
			_ => { None }
		};
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
	pub const fn blk_type_name(&self) -> &'static str {
		match self {
			BlkType::Str(_) => { "t" }
			BlkType::Int(_) => { "i" }
			BlkType::Int2(_) => { "ip2" }
			BlkType::Int3(_) => { "ip3" }
			BlkType::Long(_) => { "i64" }
			BlkType::Float(_) => { "r" }
			BlkType::Float2(_) => { "p2" }
			BlkType::Float3(_) => { "p3" }
			BlkType::Float4(_) => { "p4" }
			BlkType::Float12(_) => { "m" }
			BlkType::Bool(_) => { "b" }
			BlkType::Color(_) => { "c" }
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

impl Display for BlkType<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let value = match self {
			BlkType::Str(v) => {format!("\\{}\\", v)}
			BlkType::Int(v) => {v.to_string()}
			BlkType::Int2(v) => {format!("{}, {}", v[0], v[1])}
			BlkType::Int3(v) => {format!("{}, {}, {}", v[0], v[1], v[2])}
			BlkType::Long(v) => {v.to_string()}
			BlkType::Float(v) => {v.to_string()}
			BlkType::Float2(v) => {format!("{}, {}", v[0], v[1])}
			BlkType::Float3(v) => {format!("{}, {}, {}", v[0], v[1], v[2])}
			BlkType::Float4(v) => {format!("{}, {}, {}, {}", v[0], v[1], v[2], v[3])}
			BlkType::Float12(v) => {format!("{:?}",*v)}
			BlkType::Bool(v) => {v.to_string()}
			BlkType::Color(v) => {format!("{}, {}, {}, {}", v[3], v[2], v[1], v[0])}
		};

		write!(f, "{} = {}", self.blk_type_name(), value)
	}
}