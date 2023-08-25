use std::fmt::{Display, Formatter};

use color_eyre::{eyre::bail, Report};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
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
		write!(
			f,
			"{}",
			match self {
				HeaderType::VRFS => {
					"Base header"
				},
				HeaderType::VRFX => {
					"Extended header"
				},
			}
		)
	}
}

impl TryFrom<u32> for HeaderType {
	type Error = Report;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		return match value {
			0x73465256 => Ok(Self::VRFS),
			0x78465256 => Ok(Self::VRFX),
			_ => bail!("Unknown header type: {value:X}"),
		};
	}
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum PlatformType {
	// b'\x00\x00PC'
	Pc      = 0x43500000,

	// b'\x00iOS'
	Ios     = 0x534F6900,

	// b'\x00and'
	Android = 0x646E6100,
}

impl Display for PlatformType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				PlatformType::Pc => {
					"Pc"
				},
				PlatformType::Ios => {
					"Ios"
				},
				PlatformType::Android => {
					"Android"
				},
			}
		)
	}
}

impl TryFrom<u32> for PlatformType {
	type Error = Report;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		return match value {
			0x43500000 => Ok(Self::Pc),
			0x534F6900 => Ok(Self::Ios),
			0x646E6100 => Ok(Self::Android),
			_ => bail!("Unknown platform {value:X}"),
		};
	}
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[allow(non_camel_case_types)]
pub enum Packing {
	// ZSTD compressed and obfuscated. No digest
	ZSTD_OBFS_NOCHECK = 0x10,

	// Image in plain form. With digest
	PLAIN             = 0x20,

	// Same as ZSTD_OBFS_NOCHECK except with digest
	ZSTD_OBFS         = 0x30,
}

impl Packing {
	pub fn is_obfuscated(&self) -> bool {
		match self {
			Packing::ZSTD_OBFS_NOCHECK => true,
			Packing::PLAIN => false,
			Packing::ZSTD_OBFS => true,
		}
	}

	pub fn is_compressed(&self) -> bool {
		match self {
			Packing::ZSTD_OBFS_NOCHECK => true,
			Packing::PLAIN => false,
			Packing::ZSTD_OBFS => true,
		}
	}
}

impl TryFrom<u8> for Packing {
	type Error = Report;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		return match value {
			0x10 => Ok(Self::ZSTD_OBFS_NOCHECK),
			0x20 => Ok(Self::PLAIN),
			0x30 => Ok(Self::ZSTD_OBFS),
			_ => bail!("Unknown packing type: {value:X}"),
		};
	}
}
