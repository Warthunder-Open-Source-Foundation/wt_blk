use std::collections::HashMap;
use std::fmt::Debug;
use std::mem;
use std::str::FromStr;
use std::sync::Arc;

use serde_json::{json, Number, Value};

use crate::blk::blk_structure::BlkField;
use crate::blk::blk_type::BlkType;

impl BlkField {
	pub fn as_serde_obj(&self) -> Value {
		let mut merged = self.clone();
		merged.merge_fields();
		merged.as_serde_json().1
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
			let mut maybe_merge: HashMap<String, Vec<usize>> = HashMap::new();

			for (i, elem) in old.iter().enumerate() {
				let name = elem.as_ref().expect("Infallible").get_name();
				if let Some(dupes) = maybe_merge.get_mut(&name) {
					dupes.push(i);
				} else {
					maybe_merge.insert(name, vec![i]);
				}
			}

			maybe_merge.into_iter()
				.filter(|e| e.1.len() > 1)
				.for_each(|(key, indexes)| {
					let to_merge = indexes.iter()
						.map(|e| {
							old[*e].take().expect("Infallible")
						});
					old[indexes[0]] = Some(BlkField::Merged(Arc::new(key), to_merge.collect()));
				});
			*fields = old.into_iter().filter_map(|e|e).collect();
		}
	}
	pub fn as_serde_json(&self) -> (String, Value) {
		fn std_num<T>(num: T) -> Value
			where
				T: Debug,
		{
			Value::Number(Number::from_str(&format!("{:?}", num)).expect("Infallible"))
		}

		match self {
			BlkField::Value(k, v) => {
				(k.to_string(),
				 match v {
					 BlkType::Str(s) => { json!(s) }
					 BlkType::Int(s) => { json!(s) }
					 BlkType::Int2(s) => { json!(s) }
					 BlkType::Int3(s) => { json!(s) }
					 BlkType::Long(s) => { json!(s) }
					 BlkType::Float(s) => { std_num(s) }
					 BlkType::Float2(s) => { Value::Array(s.iter().map(|e| std_num(e)).collect()) }
					 BlkType::Float3(s) => { Value::Array(s.iter().map(|e| std_num(e)).collect()) }
					 BlkType::Float4(s) => { Value::Array(s.iter().map(|e| std_num(e)).collect()) }
					 BlkType::Float12(s) => { Value::Array(s.iter().map(|e| std_num(e)).collect()) }
					 BlkType::Bool(s) => { json!(s) }
					 BlkType::Color { r, g, b, a } => {
						 json!([r,g,b,a])
					 }
				 })
			}
			BlkField::Struct(k, v) => {
				(k.to_string(), Value::Object(serde_json::Map::from_iter(v.iter().map(|e| e.as_serde_json()))))
			}
			BlkField::Merged(k, v) => {
				(k.to_string(), Value::Array(v.iter().map(|e| e.as_serde_obj()).collect()))
			}
		}
	}
}

#[cfg(test)]
mod test {
	use serde_json::{Number, Value};

	use crate::blk::blk_structure::BlkField;
	use crate::blk::blk_type::BlkType;
	use crate::blk::util::blk_str;

	#[test]
	fn dedup_arr() {
		let blk = BlkField::Struct(blk_str("root"),
									   vec![
									   BlkField::Value(blk_str("mass"), BlkType::Float2([69.0, 42.0])),
									   BlkField::Value(blk_str("mass"), BlkType::Float2([420.0, 360.0])),
								   ]);
		let blk = blk.as_serde_obj();
		let expected = Value::Object(serde_json::Map::from_iter(vec![
			("mass".into(), Value::Array(vec![
				Value::Array(vec![Value::Number(Number::from_f64(69.0).unwrap()), Value::Number(Number::from_f64(42.0).unwrap())]),
				Value::Array(vec![Value::Number(Number::from_f64(420.0).unwrap()), Value::Number(Number::from_f64(360.0).unwrap())]),
			]))
		]));
		// println!("Found: {:#?}", blk);
		// println!("Expected: {:#?}", expected);
		assert_eq!(blk, expected);
	}

	#[test]
	fn dedup_many() {
		let blk = BlkField::Struct(blk_str("root"),
									   vec![
										   BlkField::Value(blk_str("mass"), BlkType::Float(1.0)),
										   BlkField::Value(blk_str("mass"), BlkType::Float(2.0)),
										   BlkField::Value(blk_str("mass"), BlkType::Float(3.0)),
										   BlkField::Value(blk_str("mass"), BlkType::Float(4.0)),
										   BlkField::Value(blk_str("mass"), BlkType::Float(5.0)),
										   BlkField::Value(blk_str("mass"), BlkType::Float(6.0)),
										   
										   
									   ]).as_serde_obj();
		let expected = Value::Object(serde_json::Map::from_iter(vec![
			("mass".into(), Value::Array(vec![
				Value::Number(Number::from_f64(1.0).unwrap()),
				Value::Number(Number::from_f64(2.0).unwrap()),
				Value::Number(Number::from_f64(3.0).unwrap()),
				Value::Number(Number::from_f64(4.0).unwrap()),
				Value::Number(Number::from_f64(5.0).unwrap()),
				Value::Number(Number::from_f64(6.0).unwrap()),
			]))
		]));
		// println!("Found: {:#?}", blk);
		// println!("Expected: {:#?}", expected);
		assert_eq!(blk, expected);
	}

	#[test]
	fn dedup_float() {
		let blk = BlkField::Struct(blk_str("root"),
								   vec![
									   BlkField::Value(blk_str("mass"), BlkType::Float(42.0)),
									   BlkField::Value(blk_str("mass"), BlkType::Float(69.0)),
								   ]);
		let expected = Value::Object(serde_json::Map::from_iter(vec![
			("mass".into(), Value::Array(vec![
				Value::Number(Number::from_f64(42.0).unwrap()),
				Value::Number(Number::from_f64(69.0).unwrap()),
			]))
		]));
		assert_eq!(blk.as_serde_obj(), expected);
	}

	#[test]
	fn not_everything_array() {
		let blk = BlkField::Struct(blk_str("root"),
								   vec![
									   BlkField::Value(blk_str("cheese"), BlkType::Float(42.0)),
									   BlkField::Value(blk_str("salad"), BlkType::Float(69.0)),
								   ]);
		let expected = Value::Object(serde_json::Map::from_iter(vec![
			("cheese".into(), Value::Number(Number::from_f64(42.0).unwrap())),
			("salad".into(), Value::Number(Number::from_f64(69.0).unwrap())),
		]));
		// println!("Found: {:#?}", blk.as_serde_obj());
		// println!("Expected: {:#?}", expected);
		assert_eq!(blk.as_serde_obj(), expected);
	}
}