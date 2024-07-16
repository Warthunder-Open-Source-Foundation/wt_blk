use std::{
	fmt::{Display, Formatter as StdFormatter},
	io,
	io::Write,
	sync::Arc,
};

use color_eyre::Report;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{
	ser::{Formatter, PrettyFormatter},
	Serializer as JsonSerializer,
};

use crate::blk::{
	blk_type::blk_type_id::*,
	util::{bytes_to_float, bytes_to_int, bytes_to_long, bytes_to_offset, bytes_to_uint},
};

pub type BlkString = Arc<String>;

pub mod blk_type_id {
	pub const STRING: u8 = 0x01;
	pub const INT: u8 = 0x02;
	pub const INT2: u8 = 0x07;
	pub const INT3: u8 = 0x08;
	pub const LONG: u8 = 0x0C;
	pub const FLOAT: u8 = 0x03;
	pub const FLOAT2: u8 = 0x04;
	pub const FLOAT3: u8 = 0x05;
	pub const FLOAT4: u8 = 0x06;
	pub const FLOAT12: u8 = 0x0B;
	pub const BOOL: u8 = 0x09;
	pub const COLOR: u8 = 0x0A;
}

/// Asserts the size of BlkType is constant
mod size {
	use std::mem::size_of;

	use crate::blk::blk_type::BlkType;
	const EXPECTED: usize = 16;

	const _GENERIC: usize = size_of::<BlkType>() - EXPECTED;
	const __GENERIC: usize = EXPECTED - size_of::<BlkType>();
	const _OPTIONAL: usize = size_of::<Option<BlkType>>() - EXPECTED;
	const __OPTIONAL: usize = EXPECTED - size_of::<Option<BlkType>>();
}

#[derive(Debug, PartialOrd, PartialEq, Clone, Serialize, Deserialize)]
pub enum BlkType {
	Str(BlkString),
	Int(i32),
	Int2([i32; 2]),
	Int3([i32; 3]),
	Long(i64),
	Float(f32),
	Float2([f32; 2]),
	Float3([f32; 3]),
	Float4(Box<[f32; 4]>),
	/// 3x4 Transformation matrix
	Float12(Box<[f32; 12]>),
	Bool(bool),
	Color {
		r: u8,
		g: u8,
		b: u8,
		a: u8,
	},
}

impl BlkType {
	/// Type ID as corresponding to its type_code
	/// Field is a 4 byte long region that either contains the final value or offset for the data region
	/// data_region is for non-32 bit data
	pub fn from_raw_param_info(
		type_id: u8,
		field: &[u8],
		data_region: &[u8],
		name_map: Arc<Vec<BlkString>>,
	) -> Option<Self> {

		return match type_id {
			STRING => {
				// Explanation:
				// Strings have their offset encoded as a LE integer constructed from 31 bits
				// The first bit in their field is an indicator whether or not to search in the regular data region or name map
				// The remaining bytes represent the integer
				let offset = bytes_to_uint(field)?; // Construct int from the individual bytes
				let in_nm = (offset >> 31) == 1; // Compare first bit to check where to look
				let offset = i32::MAX as u32 & offset; // Set first byte to 0
				let res: BlkString = if in_nm {
					name_map[offset as usize].clone()
				} else {
					let data_region = &data_region[(offset as usize)..];
					let mut buff = String::with_capacity(32);
					for &byte in data_region {
						if byte == 0 {
							break;
						}
						buff.push(byte as char)
					}
					Arc::from(buff)
				};

				Some(Self::Str(res))
			},
			INT => Some(Self::Int(bytes_to_int(field)?)),
			FLOAT => Some(Self::Float(bytes_to_float(field)?)),
			FLOAT2 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 8)];
				Some(Self::Float2([
					bytes_to_float(&data_region[0..4])?,
					bytes_to_float(&data_region[4..8])?,
				]))
			},
			FLOAT3 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 12)];
				Some(Self::Float3([
					bytes_to_float(&data_region[0..4])?,
					bytes_to_float(&data_region[4..8])?,
					bytes_to_float(&data_region[8..12])?,
				]))
			},
			FLOAT4 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 16)];
				Some(Self::Float4(Box::new([
					bytes_to_float(&data_region[0..4])?,
					bytes_to_float(&data_region[4..8])?,
					bytes_to_float(&data_region[8..12])?,
					bytes_to_float(&data_region[12..16])?,
				])))
			},
			INT2 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 8)];
				Some(Self::Int2([
					bytes_to_int(&data_region[0..4])?,
					bytes_to_int(&data_region[4..8])?,
				]))
			},
			INT3 => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 12)];
				Some(Self::Int3([
					bytes_to_int(&data_region[0..4])?,
					bytes_to_int(&data_region[4..8])?,
					bytes_to_int(&data_region[8..12])?,
				]))
			},
			BOOL => Some(Self::Bool(field[0] != 0)),
			COLOR => {
				// Game stores them in BGRA order
				Some(Self::Color {
					r: field[0],
					g: field[1],
					b: field[2],
					a: field[3],
				})
			},
			FLOAT12 => {
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
			},
			LONG => {
				let offset = bytes_to_offset(field)?;
				let data_region = &data_region[offset..(offset + 8)];
				Some(Self::Long(bytes_to_long(data_region)?))
			},
			_ => None,
		};
	}

	// grcov-excl-start
	pub const fn type_code(&self) -> u8 {
		match self {
			BlkType::Str(_) => STRING,
			BlkType::Int(_) => INT,
			BlkType::Int2(_) => INT2,
			BlkType::Int3(_) => INT3,
			BlkType::Long(_) => LONG,
			BlkType::Float(_) => FLOAT,
			BlkType::Float2(_) => FLOAT2,
			BlkType::Float3(_) => FLOAT3,
			BlkType::Float4(_) => FLOAT4,
			BlkType::Float12(_) => FLOAT12,
			BlkType::Bool(_) => BOOL,
			BlkType::Color { .. } => COLOR,
		}
	}

	// grcov-excl-stop

	// grcov-excl-start
	pub const fn is_inline(&self) -> bool {
		match self {
			BlkType::Str(_) => false,
			BlkType::Int(_) => true,
			BlkType::Int2(_) => false,
			BlkType::Int3(_) => false,
			BlkType::Long(_) => false,
			BlkType::Float(_) => true,
			BlkType::Float2(_) => false,
			BlkType::Float3(_) => false,
			BlkType::Float4(_) => false,
			BlkType::Float12(_) => false,
			BlkType::Bool(_) => true,
			BlkType::Color { .. } => true,
		}
	}

	// grcov-excl-stop

	// grcov-excl-start
	pub fn size_bytes(&self) -> usize {
		match self {
			BlkType::Str(inner) => inner.len(),
			BlkType::Int(_) => 4,
			BlkType::Int2(_) => 8,
			BlkType::Int3(_) => 12,
			BlkType::Long(_) => 8,
			BlkType::Float(_) => 4,
			BlkType::Float2(_) => 8,
			BlkType::Float3(_) => 12,
			BlkType::Float4(_) => 16,
			BlkType::Float12(_) => 48,
			BlkType::Bool(_) => 4,
			BlkType::Color { .. } => 4,
		}
	}

	// grcov-excl-stop

	pub const fn blk_type_name(&self) -> &'static str {
		match self {
			BlkType::Str(_) => "t",
			BlkType::Int(_) => "i",
			BlkType::Int2(_) => "ip2",
			BlkType::Int3(_) => "ip3",
			BlkType::Long(_) => "i64",
			BlkType::Float(_) => "r",
			BlkType::Float2(_) => "p2",
			BlkType::Float3(_) => "p3",
			BlkType::Float4(_) => "p4",
			BlkType::Float12(_) => "m",
			BlkType::Bool(_) => "b",
			BlkType::Color { .. } => "c",
		}
	}

	pub fn is_valid_type(t: &str) -> bool {
		matches!(
			t,
			"t" | "i" | "ip2" | "ip3" | "i64" | "r" | "p2" | "p3" | "p4" | "m" | "b" | "c"
		)
	}

	pub fn serialize_streaming(
		&self,
		w: &mut impl Write,
		ser: &mut PrettyFormatter,
	) -> Result<(), Report> {
		#[inline(always)]
		/// Writes float in format that std debug uses
		fn std_num<'b, W: Write>(_: &mut PrettyFormatter<'b>, w: &mut W, v: f32) -> io::Result<()> {
			let mut buf = ryu::Buffer::new();
			w.write_all(buf.format(v).as_bytes())?;
			Ok(())
		}
		match self {
			BlkType::Str(s) => {
				// Escapes strings according to json spec
				let mut ser = JsonSerializer::new(w);
				ser.serialize_str(s.as_str())?;
			},
			BlkType::Int(s) => {
				ser.write_i32(w, *s)?;
			},
			BlkType::Int2(s) => {
				write_generic_array(PrettyFormatter::write_i32, s.iter(), w, ser)?;
			},
			BlkType::Int3(s) => {
				write_generic_array(PrettyFormatter::write_i32, s.iter(), w, ser)?;
			},
			BlkType::Long(s) => {
				ser.write_i64(w, *s)?;
			},
			BlkType::Float(s) => {
				std_num(ser, w, *s)?;
			},
			BlkType::Float2(s) => write_generic_array(std_num, s.iter(), w, ser)?,
			BlkType::Float3(s) => write_generic_array(std_num, s.iter(), w, ser)?,
			BlkType::Float4(s) => write_generic_array(std_num, s.iter(), w, ser)?,
			BlkType::Float12(s) => {
				ser.begin_array(w)?;
				let mut begin = true;
				for chunk in s.chunks_exact(3) {
					ser.begin_array_value(w, begin)?;
					write_generic_array(std_num, chunk.iter(), w, ser)?;
					ser.end_array_value(w)?;
					begin = false;
				}
				ser.end_array(w)?;
			},
			BlkType::Bool(s) => {
				ser.write_bool(w, *s)?;
			},
			BlkType::Color { r, g, b, a } => {
				write_generic_array(PrettyFormatter::write_u8, [*r, *g, *b, *a].iter(), w, ser)?
			},
		}
		Ok(())
	}
}

fn write_generic_array<'a, 'b, T: 'a + Copy, W: Write>(
	writer: impl FnOnce(&mut PrettyFormatter<'b>, &mut W, T) -> io::Result<()> + Copy,
	mut input: impl Iterator<Item = &'a T>,
	w: &mut W,
	ser: &mut PrettyFormatter<'b>,
) -> Result<(), Report> {
	ser.begin_array(w)?;

	let mut begin = true;
	while let Some(value) = input.next() {
		ser.begin_array_value(w, begin)?;
		writer(ser, w, *value)?;
		ser.end_array_value(w)?;
		begin = false;
	}

	ser.end_array(w)?;
	Ok(())
}

impl Display for BlkType {
	fn fmt(&self, f: &mut StdFormatter<'_>) -> std::fmt::Result {
		let value = match self {
			BlkType::Str(v) => {
				format!("\"{}\"", v)
			},
			BlkType::Int(v) => v.to_string(),
			BlkType::Int2(v) => {
				format!("{}, {}", v[0], v[1])
			},
			BlkType::Int3(v) => {
				format!("{}, {}, {}", v[0], v[1], v[2])
			},
			BlkType::Long(v) => v.to_string(),
			BlkType::Float(v) => v.to_string(),
			BlkType::Float2(v) => {
				format!("{}, {}", v[0], v[1])
			},
			BlkType::Float3(v) => {
				format!("{}, {}, {}", v[0], v[1], v[2])
			},
			BlkType::Float4(v) => {
				format!("{}, {}, {}, {}", v[0], v[1], v[2], v[3])
			},
			BlkType::Float12(v) => {
				format!("{:?}", *v)
			},
			BlkType::Bool(v) => v.to_string(),
			// BGRA
			BlkType::Color { r, g, b, a } => {
				format!("{b}, {g}, {r}, {a}")
			},
		};

		write!(f, "{} = {}", self.blk_type_name(), value)
	}
}

#[cfg(test)]
mod test {
	use crate::blk::{blk_type::BlkType, util::blk_str};

	#[test]
	fn test_string() {
		let t = BlkType::Str(blk_str("yeet"));
		assert_eq!(t.to_string(), "t = \"yeet\"")
	}

	#[test]
	fn test_invalid_type() {
		assert_eq!(BlkType::is_valid_type("invalid"), false)
	}

	#[test]
	fn test_valid_types() {
		assert_eq!(
			["t", "i", "ip2", "ip3", "i64", "r", "p2", "p3", "p4", "m", "b", "c"]
				.iter()
				.all(|e| BlkType::is_valid_type(e)),
			true
		)
	}
}
