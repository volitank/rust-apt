//! There be Errors here.

use std::fmt;

use cxx::Exception;
#[doc(inline)]
pub use raw::{AptError, empty, pending_error};

#[cxx::bridge]
pub(crate) mod raw {
	/// Representation of a single Apt Error or Warning
	#[derive(Debug)]
	struct AptError {
		/// * [`true`] = Error.
		/// * [`false`] = Warning, Notice, etc.
		pub is_error: bool,
		/// The String version of the Error.
		pub msg: String,
	}

	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/error.h");

		/// Returns [`true`] if there are any pending Apt Errors.
		pub fn pending_error() -> bool;

		/// Returns [`true`] if there are no Errors or Warnings.
		pub fn empty() -> bool;

		/// Returns all Apt Errors or Warnings.
		pub fn get_all() -> Vec<AptError>;
	}
}

impl fmt::Display for AptError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.is_error {
			write!(f, "E: {}", self.msg)?;
		} else {
			write!(f, "W: {}", self.msg)?;
		}

		Ok(())
	}
}

impl std::error::Error for AptError {}

/// Struct that represents multiple apt errors and warnings.
///
/// This is essentially just a wrapper around [`Vec<AptError>`]
#[derive(Debug)]
pub struct AptErrors {
	pub(crate) ptr: Vec<AptError>,
}

impl AptErrors {
	pub fn new() -> AptErrors {
		AptErrors {
			ptr: raw::get_all(),
		}
	}
}

impl Default for AptErrors {
	fn default() -> Self { Self::new() }
}

impl fmt::Display for AptErrors {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for error in self.iter() {
			writeln!(f, "{error}")?;
		}
		Ok(())
	}
}

impl From<String> for AptErrors {
	fn from(err: String) -> Self {
		AptErrors {
			ptr: vec![AptError {
				is_error: true,
				msg: err,
			}],
		}
	}
}

impl From<Exception> for AptErrors {
	fn from(err: Exception) -> Self {
		if err.what() == "convert to AptErrors" {
			return AptErrors::new();
		}
		// The times where it's not an Apt error to be converted are slim
		AptErrors::from(err.what().to_string())
	}
}

impl From<std::io::Error> for AptErrors {
	fn from(err: std::io::Error) -> Self { AptErrors::from(err.to_string()) }
}

impl std::error::Error for AptErrors {}
