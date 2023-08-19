use std::fmt::Debug;
use std::str::FromStr;

use serde_json::{json, Map, Number, Value};
use serde_json::map::Entry;

use crate::blk::blk_structure::BlkField;
use crate::blk::blk_type::BlkType;

impl BlkField {
	pub fn as_serde_obj(&self) -> Value {
		self.as_serde_json().1
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
				let grouped_fields = v.iter().fold(Map::new(), |mut acc, field| {
					let (key, value) = field.as_serde_json();
					acc.entry(&key)
						// Merge when key exists
						.and_modify(|existing| {
							if let Value::Array(arr) = existing {
								arr.push(value.clone());
							} else {
								*existing = Value::Array(vec![existing.clone(), value.clone()]);
							}
						})
						// Insert kv pair if it doesnt
						.or_insert(value);
					acc
				});

				(k.to_string(), Value::Object(grouped_fields))
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
		let expected = Value::Object(serde_json::Map::from_iter(vec![
			("mass".into(), Value::Array(vec![
				Value::Array(vec![Value::Number(Number::from_f64(69.0).unwrap()), Value::Number(Number::from_f64(42.0).unwrap())]),
				Value::Array(vec![Value::Number(Number::from_f64(420.0).unwrap()), Value::Number(Number::from_f64(360.0).unwrap())]),
			]))
		]));
		// println!("Found: {:#?}", blk.as_serde_obj());
		// println!("Expected: {:#?}", expected);
		assert_eq!(blk.as_serde_obj(), expected);
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