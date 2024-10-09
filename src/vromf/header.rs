use wt_version::Version;

use crate::vromf::enums::{HeaderType, Packing, PlatformType};

#[derive(Debug, Default, Clone)]
pub struct Metadata {
	pub header_type: Option<HeaderType>,
	pub platform:    Option<PlatformType>,
	pub packing:     Option<Packing>,
	pub version:     Option<Version>,
}
