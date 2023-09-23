use color_eyre::Report;
use crate::blk::blk_structure::BlkField;

struct Cursor {
	at: usize,
	inner: Vec<char>,
}

impl Iterator for Cursor {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		let ret = self.inner.get(self.at).map(|e|*e);
		self.at += 1;
		ret
	}
}

pub fn deserialize_blk(input: &str) -> Result<BlkField, Report> {
	let mut c = Cursor {
		at: input.chars().count(),
		inner: input.chars().collect(),
	};
	let mut root = BlkField::new_root();
	_deserialize_blk(&mut c, &mut root)
}

enum State {
	Key,
	Type,
	ParsingValue,
}

fn _deserialize_blk(input: &mut Cursor, parent: &mut BlkField) -> Result<BlkField, Report> {
	let mut typename = String::new();
	let mut typ = String::new();
	let mut state = State::Key;

	while let Some(char) = input.next() {
		match state {
			State::Key => {
				if char != ':' {
					typename.push(char);
				} else {
					state = State::Type;
				}
			}
			State::Type => {
				if char.is_ascii_alphanumeric() {
					typ.push(char);
				} else if char == '=' {
					state = State::ParsingValue;
				}
			}
			State::ParsingValue => {

			}
		}
	}

	todo!()
}






#[cfg(test)]
mod test {
	use std::fs;
	use crate::blk::plaintext_deserialize::deserialize_blk;
	use crate::blk::test::make_strict_test;

	// #[test]
	fn test_simple() {
		let to_parse = fs::read_to_string("./samples/section_strict.blk").unwrap();

		assert_eq!(deserialize_blk(&to_parse).unwrap(), make_strict_test())
	}
}