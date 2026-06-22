use std::str::FromStr;

use serde::{Deserialize, Serialize, Serializer, Deserializer};
use wt_version::Version;

use crate::vromf::enums::{HeaderType, Packing, PlatformType};
// Kind of hacky workaround. Cant be bothered to pull in serde in wt_version just for this
fn serialize_version<S>(v: &Option<Version>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	match v {
		Some(ver) => serializer.serialize_str(&ver.to_string()),
		None => serializer.serialize_none(),
	}
}

fn deserialize_version<'de, D>(deserializer: D) -> Result<Option<Version>, D::Error>
where
	D: Deserializer<'de>,
{
	let s = Option::<String>::deserialize(deserializer)?;
	match s {
		Some(s) => Version::from_str(&s).map(Some).map_err(|_| serde::de::Error::custom("Invalid version format")),
		None => Ok(None),
	}
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Metadata {
	pub header_type: Option<HeaderType>,
	pub platform:    Option<PlatformType>,
	pub packing:     Option<Packing>,
	#[serde(default, serialize_with = "serialize_version", deserialize_with = "deserialize_version")]
	pub version:     Option<Version>,
	/// Inner container digest header flag (0x20 = no digest, 0x30 = per-file SHA1)
	#[serde(default)]
	pub digest:      Option<bool>,
}
