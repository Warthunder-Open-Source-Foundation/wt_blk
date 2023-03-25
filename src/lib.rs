#![allow(unused)] // Building the API means those warnings are just noisy TODO: Remove


#![feature(arc_unwrap_or_clone)]

pub mod binary;
pub mod output_parsing;
mod io;
mod util;
mod wrpl;
