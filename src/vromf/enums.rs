use std::fmt::{Display, Formatter};
use serde_json::ser::CharEscape::Solidus;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
pub enum HeaderType {
	// Simple header format
	VRFS = 0x73465256, // bitrepr as b"VRFs"

	// Extended header format
	VRFX = 0x78465256, // bitrepr as b"VRFx"
}

impl HeaderType {
	pub fn is_extended(self) -> bool {
		self == HeaderType::VRFX
	}
}

impl Display for HeaderType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f,"{}", match self {
			HeaderType::VRFS => {"Base header"}
			HeaderType::VRFX => {"Extended header"}
		})
	}
}

impl TryFrom<u32> for HeaderType {
	type Error = ();

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		return match value {
			0x73465256 => Ok(Self::VRFS),
			0x78465256 => Ok(Self::VRFX),
			_ => {Err(())}
		}
	}
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum PlatformType {
	// b'\x00\x00PC'
	Pc = 0x43500000,

	// b'\x00iOS'
	Ios = 0x534f6900,

	// b'\x00and'
	Android = 0x646e6100,
}

impl Display for PlatformType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f,"{}", match self {
			PlatformType::Pc => {"Pc"}
			PlatformType::Ios => {"Ios"}
			PlatformType::Android => {"Android"}
		})
	}
}

impl TryFrom<u32> for PlatformType {
	type Error = ();

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		return match value {
			0x43500000 => Ok(Self::Pc),
			0x534f6900 => Ok(Self::Ios),
			0x646e6100 => Ok(Self::Android),
			_ => {Err(())}
		}
	}
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
pub enum Packing {
	// ZSTD compressed and obfuscated. No digest
	ZSTD_OBFS_NOCHECK = 0x10,

	// Image in plain form. With digest
	PLAIN = 0x20,

	// Same as ZSTD_OBFS_NOCHECK except with digest
	ZSTD_OBFS = 0x30,
}

impl TryFrom<u8> for Packing {
	type Error = ();

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		return match value {
			0x10 => Ok(Self::ZSTD_OBFS_NOCHECK),
			0x20 => Ok(Self::PLAIN),
			0x30 => Ok(Self::ZSTD_OBFS),
			_ => {Err(())}
		}
	}
}