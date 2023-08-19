use std::fmt::Debug;
use std::str::FromStr;

use serde_json::{json, Map, Number, Value};

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
					 BlkType::Float(s) => {
						 std_num(s)
					 }
					 BlkType::Float2(s) => {
						 Value::Array(vec![std_num(s[0]), std_num(s[1])])
					 }
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
					acc.entry(key.clone())
						.and_modify(|existing| {
							if let Value::Array(arr) = existing {
								arr.push(value.clone());
							} else {
								let mut arr = Vec::new();
								arr.push(existing.clone());
								arr.push(value.clone());
								*existing = Value::Array(arr);
							}
						})
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
	use std::fs;
	use std::path::{Path, PathBuf};
	use std::str::FromStr;

	use crate::vromf::unpacker::VromfUnpacker;

	// #[test]
	fn parity_once() {
		let unpacker = VromfUnpacker::from_file((
			PathBuf::from_str("aces.vromfs.bin").unwrap(),
			include_bytes!("../../samples/aces.vromfs.bin").to_vec(),
		))
			.unwrap();
		let unpacked = unpacker
			.unpack_one_to_field(
				Path::new("gamedata/weapons/rocketguns/fr_r_550_magic_2.blk"),
			)
			.unwrap();

		let str_unpacked = serde_json::to_string_pretty(&unpacked.as_serde_obj()).unwrap();


		let reference = fs::read("./samples/magic_2_json_baseline.json").unwrap();
		let reference = String::from_utf8(reference).unwrap();
		assert_eq!(
			str_unpacked,
			reference
		);
	}
}