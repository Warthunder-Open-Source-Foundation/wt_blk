use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::Write;
use std::str::from_utf8;

/// String type that skips UTF-8 validation, simply passing it from one reader to another writer
/// Trait and function implementations are performed selectively based on direct needs
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct UnvalidatedString(Vec<u8>);

impl Debug for UnvalidatedString {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", from_utf8(self.0.as_ref())?)
	}
}

impl Display for UnvalidatedString {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", from_utf8(self.0.as_ref())?)
	}
}

impl UnvalidatedString {
	pub fn starts_with(&self, input: &[u8]) -> bool {
		self.0.starts_with(input)
	}
	pub fn replace(&self, from: &[u8], to: &[u8]) -> Self {
		let replaced = Self::replace_bytes(&self.0, from, to);
		Self(replaced)
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn push(&mut self, new: u8) {
		self.0.push(new)
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self(Vec::with_capacity(cap))
	}

	pub fn from_bytes(b: &[u8]) -> Self {
		Self(b.to_vec())
	}

	fn write_escaped(&self, w: &mut impl Write) -> io::Result<()> {
		let bytes = self.0.as_slice();
		let mut start = 0;

		for (i, &byte) in bytes.iter().enumerate() {
			let escape = ESCAPE[byte as usize];
			if escape == 0 {
				continue;
			}

			if start < i {
				w.write_all(&bytes[start..i])?;
			}

			let char_escape = CharEscape::from_escape_table(escape, byte);
			char_escape.write_char_escape(w)?;

			start = i + 1;
		}

		if start == bytes.len() {
			return Ok(());
		}

		w.write_all(&bytes[start..])
	}

	/// Writes string with escaping and paired quotes
	pub fn write_escaped_with_quotes(&self, w: &mut impl Write) -> io::Result<()> {
		w.write_all(b"\"")?;
		self.write_escaped(w)?;
		w.write_all(b"\"")?;
		Ok(())
	}

	fn replace_bytes(haystack: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
		if needle.is_empty() {
			return haystack.to_vec();
		}

		let mut result = Vec::new();
		let mut last_end = 0;

		while let Some(start) = haystack[last_end..].windows(needle.len()).position(|window| window == needle) {
			result.extend_from_slice(&haystack[last_end..last_end + start]);
			result.extend_from_slice(replacement);
			last_end += start + needle.len();
		}

		result.extend_from_slice(&haystack[last_end..]);
		result
	}
}

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const UU: u8 = b'u'; // \x00...\x1F except the ones above
const __: u8 = 0;

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
static ESCAPE: [u8; 256] = [
	//   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
	UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
	UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
	__, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
	__, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
	__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

pub enum CharEscape {
	/// An escaped quote `"`
	Quote,
	/// An escaped reverse solidus `\`
	ReverseSolidus,
	/// An escaped solidus `/`
	Solidus,
	/// An escaped backspace character (usually escaped as `\b`)
	Backspace,
	/// An escaped form feed character (usually escaped as `\f`)
	FormFeed,
	/// An escaped line feed character (usually escaped as `\n`)
	LineFeed,
	/// An escaped carriage return character (usually escaped as `\r`)
	CarriageReturn,
	/// An escaped tab character (usually escaped as `\t`)
	Tab,
	/// An escaped ASCII plane control character (usually escaped as
	/// `\u00XX` where `XX` are two hex characters)
	AsciiControl(u8),
}

impl CharEscape {
	#[inline]
	fn from_escape_table(escape: u8, byte: u8) -> CharEscape {
		match escape {
			self::BB => CharEscape::Backspace,
			self::TT => CharEscape::Tab,
			self::NN => CharEscape::LineFeed,
			self::FF => CharEscape::FormFeed,
			self::RR => CharEscape::CarriageReturn,
			self::QU => CharEscape::Quote,
			self::BS => CharEscape::ReverseSolidus,
			self::UU => CharEscape::AsciiControl(byte),
			_ => unreachable!(),
		}
	}

	#[inline]
	fn write_char_escape<W>(&self, writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
	{
		use self::CharEscape::*;

		let s = match self {
			Quote => b"\\\"",
			ReverseSolidus => b"\\\\",
			Solidus => b"\\/",
			Backspace => b"\\b",
			FormFeed => b"\\f",
			LineFeed => b"\\n",
			CarriageReturn => b"\\r",
			Tab => b"\\t",
			AsciiControl(byte) => {
				static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
				let bytes = &[
					b'\\',
					b'u',
					b'0',
					b'0',
					HEX_DIGITS[(byte >> 4) as usize],
					HEX_DIGITS[(byte & 0xF) as usize],
				];
				return writer.write_all(bytes);
			}
		};

		writer.write_all(s)
	}
}