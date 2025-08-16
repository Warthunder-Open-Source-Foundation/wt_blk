use std::{io::Write, mem};

use color_eyre::Report;
use foldhash::HashMapExt;
use serde_json::ser::{Formatter, PrettyFormatter};
use smallvec::{smallvec, SmallVec};

use crate::blk::{blk_string::BlkString, blk_structure::BlkField};

impl BlkField {
	/// Merges duplicate keys in struct fields into the Merged array variant
	pub fn merge_fields(&mut self) -> Result<(), Report> {
		if let BlkField::Struct(_, fields) = self {
			// Recurse first
			for field in fields.iter_mut() {
				field.merge_fields()?;
			}

			let mut to_merge = mem::take(fields)
				.into_iter()
				.map(|field| Some(field))
				.collect::<Vec<_>>(); // Yoink the old vector to merge its fields

			type MergeIndex = u16;

			// Key: Field-name, Value: Indexes of duplicates found
			let mut duplicates: foldhash::HashMap<BlkString, SmallVec<[MergeIndex; 4]>> =
				foldhash::HashMap::with_capacity(to_merge.len());

			for (i, elem) in to_merge.iter().enumerate() {
				let name = elem.as_ref().expect("Infallible").get_name();
				// Saving some space, as there won't be more than 2^16 fields
				let i = i.try_into()?;
				duplicates
					.entry(name)
					.and_modify(|e| e.push(i))
					.or_insert_with(|| smallvec![i]);
			}

			duplicates
				.into_iter()
				// Skip any key with only one occurrence
				.filter(|e| e.1.len() > 1)
				.for_each(|(key, indexes)| {
					let merge_into = indexes[0];
					let others = indexes
						.into_iter()
						.map(|e| to_merge[e as usize].take().expect("Infallible"))
						.collect();
					to_merge[merge_into as usize] = Some(BlkField::Merged(key, others));
				});
			*fields = to_merge.into_iter().filter_map(|e| e).collect();
		}
		Ok(())
	}

	pub fn as_serde_json(&self) -> Result<Vec<u8>, Report> {
		let mut res = vec![];
		self.as_serde_json_streaming(&mut res)?;
		Ok(res)
	}

	pub fn as_serde_json_string(&self) -> Result<String, Report> {
		Ok(String::from_utf8(self.as_serde_json()?)?)
	}

	/// Ensure the writer is buffered, since this will call write many many times
	pub fn as_serde_json_streaming(&self, w: &mut impl Write) -> Result<(), Report> {
		// let mut ser = PrettyFormatter::with_indent(b"\t");
		let mut ser = PrettyFormatter::new();
		self._as_serde_json_streaming(w, &mut ser, true, true, false)?;
		w.flush()?;
		Ok(())
	}

	fn _as_serde_json_streaming(
		&self,
		w: &mut impl Write,
		ser: &mut PrettyFormatter,
		is_root: bool,
		is_first: bool,
		in_merging_array: bool,
	) -> Result<(), Report> {
		match self {
			BlkField::Value(k, v) => {
				if !in_merging_array {
					ser.begin_object_key(w, is_first)?;
					ser.begin_string(w)?;
					ser.write_string_fragment(w, k.as_ref())?;
					ser.end_string(w)?;
					ser.end_object_key(w)?;

					ser.begin_object_value(w)?;
				}
				v.serialize_streaming(w, ser)?;

				if !in_merging_array {
					ser.end_object_value(w)?;
				}
			},
			BlkField::Struct(k, v) => {
				// Skip over object key when root or merging an array
				if !is_root && !in_merging_array {
					ser.begin_object_key(w, is_first)?;
					ser.begin_string(w)?;
					ser.write_string_fragment(w, k.as_ref())?;
					ser.end_string(w)?;
					ser.end_object_key(w)?;
					ser.begin_object_value(w)?;
				}
				// Empty objects should not have newlines in them, so we simply skip them
				if !v.is_empty() {
					ser.begin_object(w)?;
					let mut is_first = true;
					for value in v {
						value._as_serde_json_streaming(w, ser, false, is_first, false)?;
						is_first = false;
					}
					ser.end_object_value(w)?;
					ser.end_object(w)?;

				// Instead we just write brackets without a newline in them
				} else {
					ser.write_string_fragment(w, "{}")?;
				}
			},
			BlkField::Merged(k, v) => {
				ser.begin_object_key(w, is_first)?;
				ser.begin_string(w)?;
				ser.write_string_fragment(w, k.as_ref())?;
				ser.end_string(w)?;
				ser.end_object_key(w)?;
				ser.begin_object_value(w)?;

				ser.begin_array(w)?;
				let mut is_first = true;
				for value in v {
					ser.begin_array_value(w, is_first)?;
					value._as_serde_json_streaming(w, ser, false, is_first, true)?;
					is_first = false;
					ser.end_array_value(w)?;
				}
				ser.end_array(w)?;
			},
		}
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use std::fs;

	use crate::blk::{
		blk_string::blk_str,
		blk_structure::BlkField,
		blk_type::BlkType,
		make_strict_test,
	};

	#[test]
	fn streaming() {
		let blk = make_strict_test();
		// println!("Found: {:#?}", blk.as_serde_obj());
		// println!("Expected: {:#?}", expected);
		let mut buf = vec![];
		blk.as_serde_json_streaming(&mut buf).unwrap();
		assert_eq!(
			String::from_utf8(buf).unwrap(),
			fs::read_to_string("./samples/expected.json").unwrap()
		);
	}

	#[test]
	fn streaming_merge() {
		let mut blk = make_strict_test();
		blk.insert_field(BlkField::Value(blk_str("int"), BlkType::Int(420)))
			.unwrap();
		blk.merge_fields().unwrap();
		let mut buf = vec![];
		blk.as_serde_json_streaming(&mut buf).unwrap();
		assert_eq!(
			String::from_utf8(buf).unwrap(),
			fs::read_to_string("./samples/expected_merged.json").unwrap()
		);
	}

	#[test]
	fn streaming_empty() {
		let blk = BlkField::new_root();
		let mut buf = vec![];
		blk.as_serde_json_streaming(&mut buf).unwrap();
		assert_eq!(String::from_utf8(buf).unwrap(), "{}");
	}

	#[test]
	fn consistency() {
		let sample = make_strict_test();
		let s = sample.as_serde_json_string().unwrap();
		assert_eq!(s, fs::read_to_string("./samples/expected.json").unwrap());
	}
}
