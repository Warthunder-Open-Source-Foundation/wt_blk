#![feature(array_chunks)]
#![feature(iter_array_chunks)]

/// Low-level functions for BLK file format, for high level API use the [`vromf`] module
pub mod blk;

/// Misc. utility functions for the DXP and GRP file-format
pub mod dxp_and_grp;

/// General utility functions used in the entire crate
mod util;

/// High-level API for unpacking entire Vromf archives
pub mod vromf;

/// Experimental WRPL unpacking (WIP)
mod wrpl;

/// Performance instrumentation, using the stamp! macro
#[allow(unused)]
mod perf_instrumentation;
mod repacker_util;
