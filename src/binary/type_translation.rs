use std::collections::HashMap;
use lazy_static::lazy_static;

// Private const represenation of types
const TYPE_DEF: &[TypeDef] = &[
	TypeDef {
		name: "Str",
		type_id: TypeId::Str,
		length: 0,
		is_short: false,
	},
	TypeDef {
		name: "Int",
		type_id: TypeId::Int,
		length: 4,
		is_short: true,
	},
	TypeDef {
		name: "Float",
		type_id: TypeId::Float,
		length: 4,
		is_short: true,
	},
	TypeDef {
		name: "Float2",
		type_id: TypeId::Float2,
		length: 8,
		is_short: false,
	},
	TypeDef {
		name: "Float3",
		type_id: TypeId::Float3,
		length: 12,
		is_short: false,
	},
	TypeDef {
		name: "Float4",
		type_id: TypeId::Float4,
		length: 16,
		is_short: false,
	},
	TypeDef {
		name: "Int2",
		type_id: TypeId::Int2,
		length: 8,
		is_short: false,
	},
	TypeDef {
		name: "Int3",
		type_id: TypeId::Int3,
		length: 12,
		is_short: false,
	},
	TypeDef {
		name: "Bool",
		type_id: TypeId::Bool,
		length: 4,
		is_short: true,
	},
	TypeDef {
		name: "Color",
		type_id: TypeId::Color,
		length: 4,
		is_short: true,
	},
	TypeDef {
		name: "Float12",
		type_id: TypeId::Float12,
		length: 48,
		is_short: false,
	},
	TypeDef {
		name: "Long",
		type_id: TypeId::Long,
		length: 8,
		is_short: false,
	},
];

lazy_static! {
    pub static ref TYPE_MAP: HashMap<u8, TypeDef<'static>> = {
		let mut map = HashMap::new();
		for i in TYPE_DEF.iter() {
			map.insert(i.type_id as u8, *i);
		}

		map
	};
}


#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum TypeId {
	Str = 0x01,
	Int = 0x02,
	Float = 0x03,
	Float2 = 0x04,
	Float3 = 0x05,
	Float4 = 0x06,
	Int2 = 0x07,
	Int3 = 0x08,
	Bool = 0x09,
	Color = 0x0a,
	Float12 = 0x0b,
	Long = 0x0c,
}

#[derive(Copy, Clone, Hash)]
pub struct TypeDef<'a> {
	pub name: &'a str,

	pub type_id: TypeId,

	// stored in multiples of 8, get bitsize with getter `bit_size`
	pub length: usize,

	// means they are stored in-place after their definition

	pub is_short: bool,
}

impl TypeDef<'_> {
	pub fn bit_size(&self) -> usize {
		if self.type_id != TypeId::Str {
			self.length * 8
		} else {
			unimplemented!()
		}
	}
}
