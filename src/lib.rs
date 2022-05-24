//! rust-apt provides bindings to `libapt-pkg`.
//! The goal is to eventually have all of the functionality of `python-apt`
//!
//! The source repository is https://gitlab.com/volian/rust-apt
//! For more information please see the readme in the source code.

pub mod cache;
mod package;
mod raw;
