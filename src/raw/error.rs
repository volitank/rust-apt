use std::fmt;

use cxx::Exception;

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {
	impl Vec<AptError> {}
	#[derive(Debug)]
	struct AptError {
		pub is_error: bool,
		pub msg: String,
	}

	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/error.h");

		pub fn pending_error() -> bool;

		pub fn empty() -> bool;

		pub fn get_all() -> Vec<AptError>;
	}
}

impl fmt::Display for raw::AptError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.is_error {
			write!(f, "E: {}", self.msg)?;
		} else {
			write!(f, "W: {}", self.msg)?;
		}

		Ok(())
	}
}

impl std::error::Error for raw::AptError {}

#[derive(Debug)]
pub struct AptErrors {
	errors: Vec<raw::AptError>,
}

impl AptErrors {
	pub fn new() -> AptErrors {
		AptErrors {
			errors: raw::get_all(),
		}
	}

	pub fn errors(&self) -> &Vec<raw::AptError> { &self.errors }
}

impl Default for AptErrors {
	fn default() -> Self { Self::new() }
}

impl fmt::Display for AptErrors {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for error in &self.errors {
			writeln!(f, "{error}")?;
		}
		Ok(())
	}
}

impl From<String> for AptErrors {
	fn from(err: String) -> Self {
		AptErrors {
			errors: vec![raw::AptError {
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
