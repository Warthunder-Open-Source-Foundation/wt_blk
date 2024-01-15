use std::{collections::HashMap, mem, str::FromStr, sync::Arc};

use serde_json::{json, Number, Value};

use crate::blk::{blk_structure::BlkField, blk_type::BlkType};
use crate::blk::blk_type::BlkString;

impl BlkField {
	pub fn as_serde_obj(&mut self, should_override: bool) -> Value {
		self.merge_fields();
		if should_override {
			self.apply_overrides();
		}
		self.as_serde_json(should_override).1
	}

	/// Merges duplicate keys in struct fields into the Merged array variant
	pub fn merge_fields(&mut self) {
		if let BlkField::Struct(_, fields) = self {
			// Recurse first
			for field in fields.iter_mut() {
				field.merge_fields();
			}

			let mut old = mem::take(fields)
				.into_iter()
				.map(|field| Some(field))
				.collect::<Vec<_>>(); // Yoink the old vector to merge its fields

			// Key: Field-name, Value: Indexes of duplicates found
			let mut maybe_merge: HashMap<BlkString, Vec<usize>> = HashMap::with_capacity(old.len());

			for (i, elem) in old.iter().enumerate() {
				let name = elem.as_ref().expect("Infallible").get_name();
				if let Some(dupes) = maybe_merge.get_mut(name.as_ref()) {
					dupes.push(i);
				} else {
					maybe_merge.insert(name.clone(), vec![i]);
				}
			}

			maybe_merge
				.into_iter()
				.filter(|e| e.1.len() > 1)
				.for_each(|(key, indexes)| {
					let first_element = indexes[0];
					let to_merge = indexes
						.into_iter()
						.map(|e| old[e].take().expect("Infallible"))
						.collect();
					old[first_element] = Some(BlkField::Merged(Arc::from(key), to_merge));
				});
			*fields = old.into_iter().filter_map(|e| e).collect();
		}
	}

	/// Returns key as string, and value as `serde_json::Value`
	pub fn as_serde_json(&mut self, apply_overrides: bool) -> (String, Value) {
		#[inline(always)]
		fn std_num(num: f32) -> Value {
			Value::Number(Number::from_str(&format!("{:?}", num)).expect("Infallible"))
		}

		match self {
			BlkField::Value(k, v) => (
				k.to_string(),
				match v {
					BlkType::Str(s) => {
						json!(s)
					}
					BlkType::Int(s) => {
						json!(s)
					}
					BlkType::Int2(s) => {
						json!(s)
					}
					BlkType::Int3(s) => {
						json!(s)
					}
					BlkType::Long(s) => {
						json!(s)
					}
					BlkType::Float(s) => std_num(*s as f32),
					BlkType::Float2(s) => Value::Array(s.iter().map(|e| std_num(*e)).collect()),
					BlkType::Float3(s) => Value::Array(s.iter().map(|e| std_num(*e)).collect()),
					BlkType::Float4(s) => Value::Array(s.iter().map(|e| std_num(*e)).collect()),
					BlkType::Float12(s) => Value::Array(
						s.array_chunks::<3>()
							.map(|e| e.iter().map(|e| std_num(*e)).collect())
							.collect(),
					),
					BlkType::Bool(s) => {
						json!(s)
					}
					BlkType::Color { r, g, b, a } => {
						json!([r, g, b, a])
					}
				},
			),
			BlkField::Struct(k, v) => (
				k.to_string(),
				Value::Object(serde_json::Map::from_iter(
					v.iter_mut().map(|e| e.as_serde_json(apply_overrides)),
				)),
			),
			BlkField::Merged(k, v) => (
				k.to_string(),
				Value::Array(v.iter_mut().map(|e| e.as_serde_obj(apply_overrides)).collect()),
			),
		}
	}
}

#[cfg(test)]
mod test {
	use std::fs;
	use serde_json::{Number, Value};

	use crate::blk::{blk_structure::BlkField, blk_type::BlkType, make_strict_test, util::blk_str};

	#[test]
	fn dedup_arr() {
		let mut blk = BlkField::Struct(
			blk_str("root"),
			vec![
				BlkField::Value(blk_str("mass"), BlkType::Float2([69.0, 42.0])),
				BlkField::Value(blk_str("mass"), BlkType::Float2([420.0, 360.0])),
			],
		);
		let blk = blk.as_serde_obj(true);
		let expected = Value::Object(serde_json::Map::from_iter(vec![(
			"mass".into(),
			Value::Array(vec![
				Value::Array(vec![
					Value::Number(Number::from_f64(69.0).unwrap()),
					Value::Number(Number::from_f64(42.0).unwrap()),
				]),
				Value::Array(vec![
					Value::Number(Number::from_f64(420.0).unwrap()),
					Value::Number(Number::from_f64(360.0).unwrap()),
				]),
			]),
		)]));
		// println!("Found: {:#?}", blk);
		// println!("Expected: {:#?}", expected);
		assert_eq!(blk, expected);
	}

	#[test]
	fn dedup_many() {
		let blk = BlkField::Struct(
			blk_str("root"),
			vec![
				BlkField::Value(blk_str("mass"), BlkType::Float(1.0)),
				BlkField::Value(blk_str("mass"), BlkType::Float(2.0)),
				BlkField::Value(blk_str("mass"), BlkType::Float(3.0)),
				BlkField::Value(blk_str("mass"), BlkType::Float(4.0)),
				BlkField::Value(blk_str("mass"), BlkType::Float(5.0)),
				BlkField::Value(blk_str("mass"), BlkType::Float(6.0)),
			],
		)
			.as_serde_obj(true);
		let expected = Value::Object(serde_json::Map::from_iter(vec![(
			"mass".into(),
			Value::Array(vec![
				Value::Number(Number::from_f64(1.0).unwrap()),
				Value::Number(Number::from_f64(2.0).unwrap()),
				Value::Number(Number::from_f64(3.0).unwrap()),
				Value::Number(Number::from_f64(4.0).unwrap()),
				Value::Number(Number::from_f64(5.0).unwrap()),
				Value::Number(Number::from_f64(6.0).unwrap()),
			]),
		)]));
		// println!("Found: {:#?}", blk);
		// println!("Expected: {:#?}", expected);
		assert_eq!(blk, expected);
	}

	#[test]
	fn dedup_float() {
		let mut blk = BlkField::Struct(
			blk_str("root"),
			vec![
				BlkField::Value(blk_str("mass"), BlkType::Float(42.0)),
				BlkField::Value(blk_str("mass"), BlkType::Float(69.0)),
			],
		);
		let expected = Value::Object(serde_json::Map::from_iter(vec![(
			"mass".into(),
			Value::Array(vec![
				Value::Number(Number::from_f64(42.0).unwrap()),
				Value::Number(Number::from_f64(69.0).unwrap()),
			]),
		)]));
		assert_eq!(blk.as_serde_obj(true), expected);
	}

	#[test]
	fn not_everything_array() {
		let mut blk = BlkField::Struct(
			blk_str("root"),
			vec![
				BlkField::Value(blk_str("cheese"), BlkType::Float(42.0)),
				BlkField::Value(blk_str("salad"), BlkType::Float(69.0)),
			],
		);
		let expected = Value::Object(serde_json::Map::from_iter(vec![
			(
				"cheese".into(),
				Value::Number(Number::from_f64(42.0).unwrap()),
			),
			(
				"salad".into(),
				Value::Number(Number::from_f64(69.0).unwrap()),
			),
		]));
		// println!("Found: {:#?}", blk.as_serde_obj());
		// println!("Expected: {:#?}", expected);
		assert_eq!(blk.as_serde_obj(true), expected);
	}

	#[test]
	fn int_without_dot() {
		let mut blk = BlkField::Struct(
			blk_str("root"),
			vec![
				BlkField::Value(blk_str("salad"), BlkType::Int(69)),
			],
		);
		let expected = Value::Object(serde_json::Map::from_iter(vec![
			(
				"salad".into(),
				Value::Number(Number::from_f64(69.0).unwrap()),
			),
		]));
		// println!("Found: {:#?}", blk.as_serde_obj());
		// println!("Expected: {:#?}", expected);
		assert_ne!(blk.as_serde_obj(true), expected);
	}

	#[test]
	fn int_array_without_dot() {
		let mut blk = BlkField::Struct(
			blk_str("root"),
			vec![
				BlkField::Value(blk_str("salad"), BlkType::Int2([69, 420])),
			],
		);
		let expected = Value::Object(serde_json::Map::from_iter(vec![
			(
				"salad".into(),
				Value::Array(vec![Value::Number(Number::from_f64(69.0).unwrap()), Value::Number(Number::from_f64(420.0).unwrap())]),
			),
		]));
		// println!("Found: {:#?}", blk.as_serde_obj());
		// println!("Expected: {:#?}", expected);
		assert_ne!(blk.as_serde_obj(true), expected);
	}

	#[test]
	fn consistency() {
		let mut sample = make_strict_test();
		assert_eq!(serde_json::to_string_pretty(&sample.as_serde_obj(true)).unwrap(), fs::read_to_string("./samples/expected.json").unwrap());
	}
}
