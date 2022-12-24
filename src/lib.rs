//! rust-apt provides bindings to `libapt-pkg`.
//! The goal is to eventually have all of the functionality of `python-apt`
//!
//! The source repository is <https://gitlab.com/volian/rust-apt>
//! For more information please see the readme in the source code.
//!
//! Each module has a `raw` submodule containing c++ bindings to `libapt-pkg`
//!
//! These are safe to use in terms of memory,
//! but may cause segfaults if you do something wrong.
//!
//! If you find a way to segfault without using the `libapt-pkg` bindings
//! directly, please report this as a bug.

#[macro_use]
pub mod raw;
pub mod cache;
pub mod config;
pub mod depcache;
pub mod macros;
pub mod package;
pub mod records;
pub mod tagfile;
pub mod util;
