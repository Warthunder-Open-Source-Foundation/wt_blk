#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FormattingConfiguration {
	/// Char used to indent n times
	/// Examples:
	/// ('\t', 1) will indent using tabs, once for each level of indentation
	/// (' ', 4) will indent using spaces, four times for each level of indentation
	pub indent_char: (char, usize),

	/// Should a float that is a real number (integer) lose their trailing 0
	/// Example:
	/// 	True:
	/// 	5.0 => 5
	/// 	5.5 => 5.5
	/// 	False:
	/// 	5.0 => 5.0
	/// 	5.5 => 5.5
	pub natural_float_truncate_0: bool,

	/// Wraps entire output in one pair of curly brackets, therefore indenting the entire file once
	pub global_curly_bracket: bool,

	/// Beginning of objects may be prefixed with a colon before opening the brackets
	pub object_colon: bool,
}

impl FormattingConfiguration {
	pub const GSZABI_REPO: FormattingConfiguration = Self {
		indent_char: (' ', 2),
		natural_float_truncate_0: true,
		global_curly_bracket: true,
		object_colon: true,
	};
	pub fn indent(self, level: usize) -> String {
		self.indent_char.0.to_string().repeat(level * self.indent_char.1)
	}
}