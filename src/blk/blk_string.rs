use std::ops::Deref;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Range};
use std::sync::Arc;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone)]
/// Efficient string wrapper for this specific application
pub struct BlkString {
	inner: Arc<RefBlkString>,
}

#[derive(Clone)]
enum RefBlkString {
	Referenced {
		from: Arc<String>,
		range: Range<usize>,
	},
	Owned {
		inner: String,
	}
}

impl BlkString {
	/// Creates Owned BlkString allocating the string if neccesary
	pub fn new(inner: impl Into<String>) -> Self {
		Self {
			inner: Arc::new(RefBlkString::Owned { inner: inner.into() }),
		}
	}

	pub fn from_range(memory: Arc<String>, range: Range<usize>) -> Self {
		Self { inner: Arc::new(RefBlkString::Referenced { from: memory, range }) }
	}

	/// This function should exclusively be used for accessing the string contents, as future
	/// optimizations will change the internals
	pub fn as_str(&self) -> &str {
		match self.inner.deref() {
			RefBlkString::Referenced { from, range } => {
				from[range]
			}
			RefBlkString::Owned { inner } => inner.as_str()
		}
	}
}

impl Deref for BlkString {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl Borrow<str> for BlkString {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl Display for BlkString {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl From<String> for BlkString {
	fn from(value: String) -> Self {
		Self::new(value)
	}
}

/// Wrapper for quickly creating Arced string
// TODO: use proper constructor instead
pub fn blk_str(s: impl Into<String>) -> BlkString {
	BlkString::from(s.into())
}

impl Serialize for BlkString {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer
	{
		serializer.serialize_str(self.as_str())
	}
}


impl<'de> Deserialize<'de> for BlkString {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		Ok(BlkString {
			inner: Arc::new(s),
		})
	}
}

impl Debug for BlkString {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.as_str().fmt(f)
	}
}

impl PartialEq for BlkString {
	fn eq(&self, other: &Self) -> bool {
		self.as_str().eq(other.as_str())
	}
}

impl Eq for BlkString{}

impl Hash for BlkString {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_str().hash(state);
	}
}

impl PartialOrd for BlkString {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.as_str().partial_cmp(other.as_str())
	}
}

#[cfg(test)]
mod test {
	use crate::blk::blk_string::BlkString;

	#[test]
	pub fn serialize() {
		let sample = BlkString::from("test".to_owned());
		let ser  = serde_json::to_string_pretty(&sample).unwrap();

		assert_eq!(ser.as_str(), "\"test\"");
	}

	#[test]
	pub fn deserialize() {
		assert_eq!(BlkString::from("test".to_owned()), serde_json::from_str("\"test\"").unwrap());
	}
}