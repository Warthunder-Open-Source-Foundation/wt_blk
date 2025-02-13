#![allow(rustdoc::private_intra_doc_links)]

//! # Virtual Read Only Memory Filesystem - VROMFS
//!
//! vromfs are an archive format similar to ZIP, bundling a collection of files into a compressible container.
//! The files are usually suffixed with "vromfs.bin".
//! The file consistent of two core parts, an initial outer "shell" and an inner payload. These will be referred to as "binary container" and "inner container".
//! The binary container docs can be found here [`crate::vromf::binary_container`].
//! The inner container docs can be found here [`crate::vromf::inner_container`].

pub mod de_obfuscation;
mod enums;
mod util;

/// This module unpacks the "outer" shell of the vromf image
pub(crate) mod binary_container;

pub(crate) mod file;
mod header;
pub(crate) mod inner_container;
#[cfg(test)]
mod test;
mod unpacker;

pub use file::File;
pub use unpacker::{BlkOutputFormat, VromfUnpacker, ZipFormat};
