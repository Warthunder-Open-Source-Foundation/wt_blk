pub mod error;

use crate::output_parsing::error::WTBlkError;
use serde_json::{Map, Value};

#[derive(Debug)]
pub struct WTBlk {
    inner: Value,
}

impl WTBlk {
    pub fn new(str: &str) -> Result<Self, WTBlkError> {
        let inner: Value = serde_json::from_str(str)?;
        Ok(Self { inner })
    }

    /// Returns direct reference result after parsing pointer
    pub fn pointer<'a>(&'a self, pointer: &'a str) -> Result<&Value, WTBlkError> {
        self.inner
            .pointer(pointer)
            .ok_or_else(|| WTBlkError::NoSuchValue(pointer.to_owned()))
    }

    /// Returns a direct mutable reference to the inner serde struct
    pub fn inner(&mut self) -> &mut Value {
        &mut self.inner
    }

    /// Returns boolean value
    pub fn bool<'a>(&'a self, pointer: &'a str) -> Result<bool, WTBlkError> {
        let res = self.pointer(pointer)?;
        res.as_bool()
            .ok_or_else(|| WTBlkError::Parse(pointer.to_owned(), res.to_string()))
    }

    /// Returns 64-bit floating point
    pub fn float<'a>(&'a self, pointer: &'a str) -> Result<f64, WTBlkError> {
        let res = self.pointer(pointer)?;
        res.as_f64()
            .ok_or_else(|| WTBlkError::Parse(pointer.to_owned(), res.to_string()))
    }

    /// Returns signed 64-bit integer, cast if needed
    pub fn int<'a>(&'a self, pointer: &'a str) -> Result<i64, WTBlkError> {
        let res = self.pointer(pointer)?;
        res.as_i64()
            .ok_or_else(|| WTBlkError::Parse(pointer.to_owned(), res.to_string()))
    }

    /// Returns string value
    pub fn str<'a>(&'a self, pointer: &'a str) -> Result<&str, WTBlkError> {
        let res = self.pointer(pointer)?;
        res.as_str()
            .ok_or_else(|| WTBlkError::Parse(pointer.to_owned(), res.to_string()))
    }

    /// Returns KV for all following child items
    pub fn objects<'a>(&'a self, pointer: &'a str) -> Result<&Map<String, Value>, WTBlkError> {
        let res = self.pointer(pointer)?;
        res.as_object()
            .ok_or_else(|| WTBlkError::Parse(pointer.to_owned(), res.to_string()))
    }

    /// Returns field array
    pub fn array<'a>(&'a self, pointer: &'a str) -> Result<&Vec<Value>, WTBlkError> {
        let res = self.pointer(pointer)?;
        res.as_array()
            .ok_or_else(|| WTBlkError::Parse(pointer.to_owned(), res.to_string()))
    }
}
