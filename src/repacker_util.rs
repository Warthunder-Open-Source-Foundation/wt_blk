use std::ops::Range;
use color_eyre::Report;
use color_eyre::eyre::{ensure, ContextCompat};
use std::io::{Cursor, Write};

pub struct Buffer {
	pub inner: Cursor<Vec<u8>>
}

impl Buffer {
	pub fn u32(&mut self) -> Result<RefRange<4, u32>, Report> {
		let start = self.inner.position() as _;
		self.inner.write_all(&[0; 4])?;
		Ok(RefRange {
			value: None,
			serializer: |t| u32::to_le_bytes(t),
			range: start..start + 4,
			written: false,
		})
	}

	pub fn u64(&mut self) -> Result<RefRange<8, u64>, Report> {
		let start = self.inner.position() as _;
		self.inner.write_all(&[0; 8])?;
		Ok(RefRange {
			value: None,
			serializer: |t| u64::to_le_bytes(t),
			range: start..start + 8,
			written: false,
		})
	}

	pub fn pad_zeroes<const N: usize>(&mut self) -> Result<(), Report> {
		self.inner.write_all(&[0; N])?;
		Ok(())
	}

	pub fn align_to_multiple_of_16(&mut self) -> Result<(), Report> {
		let pos = self.inner.position();
		let target = (pos + 15) & !15;
		for _ in pos..target {
			self.inner.write(&[0; 1])?;
		}
		Ok(())
	}
}

#[must_use]
pub struct RefRange<const N: usize, T> {
	value: Option<T>,
	serializer: fn(T) -> [u8; N],
	range: Range<usize>,
	written: bool,
}

impl <const N: usize, T: Copy>RefRange<N, T> {
	pub fn write_to(mut self, buf: &mut Buffer) -> Result<(), Report> {
		let ser = (self.serializer)(self.value.context("T was not initialized")?);
		ensure!(ser.len() == self.range.len(), "Serialized length mismatches with assigned range");
		buf.inner.get_mut().get_mut(self.range.clone()).context("todo")?.copy_from_slice(&ser);
		self.written = true;
		Ok(())
	}

	// pub fn from(value: T, range: Range<usize>, serializer: fn(T) -> [u8; N]) -> Self {
	// 	Self {
	// 		value: Some(value),
	// 		serializer,
	// 		range,
	// 		written: false,
	// 	}
	// }

	pub fn set(&mut self, v: T) {
		self.value = Some(v);
	}

	pub fn set_write(mut self, t: T, dst: &mut Buffer) -> Result<(), Report> {
		self.set(t);
		self.write_to(dst)?;
		Ok(())
	}

	// pub fn after<const X: usize, Y>(mut self, other: &RefRange<X, Y>) -> Self {
	// 	self.range.start = other.range.end;
	// 	self.range.end = self.range.start + N;
	// 	self
	// }

	// pub fn value_mut(&mut self) -> Option<&mut T> {
	// 	self.value.as_mut()
	// }
}

impl<const N: usize, T> Drop for RefRange<N, T> {
	fn drop(&mut self) {
		assert!(self.written, "Dropped RefRange without writing it")
	}
}