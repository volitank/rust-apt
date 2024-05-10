//! Contains Cache related structs.

use cxx::UniquePtr;

use super::error::raw::AptError;
use super::error::AptErrors;
use super::package::{IntoRawIter, RawPackage};

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {

	impl UniquePtr<PkgRecords> {}
	impl UniquePtr<IndexFile> {}

	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/cache.h");
		type PkgCacheFile;

		type PkgFileIterator = crate::raw::package::raw::PkgFileIterator;
		type IndexFile;

		type PkgIterator = crate::raw::package::raw::PkgIterator;
		type VerIterator = crate::raw::package::raw::VerIterator;
		type SourceURI = crate::raw::package::raw::SourceURI;

		type PkgRecords = crate::raw::records::raw::PkgRecords;
		type PkgDepCache = crate::raw::depcache::raw::PkgDepCache;

		type DynAcquireProgress = crate::raw::progress::raw::DynAcquireProgress;

		pub fn u_create_cache(deb_files: &[String]) -> Result<UniquePtr<PkgCacheFile>>;

		pub fn u_update(self: &PkgCacheFile, progress: &mut DynAcquireProgress) -> Result<()>;

		/// Returns an iterator of SourceURIs.
		///
		/// These are the files that `apt update` will fetch.
		pub fn source_uris(self: &PkgCacheFile) -> Vec<SourceURI>;

		pub fn create_depcache(self: &PkgCacheFile) -> UniquePtr<PkgDepCache>;

		pub fn create_records(self: &PkgCacheFile) -> UniquePtr<PkgRecords>;

		/// The priority of the Version as shown in `apt policy`.
		pub fn priority(self: &PkgCacheFile, version: &VerIterator) -> i32;

		/// Lookup the IndexFile of the Package file
		pub fn find_index(self: &PkgCacheFile, file: &PkgFileIterator) -> UniquePtr<IndexFile>;

		/// Return true if the PackageFile is trusted.
		pub fn is_trusted(self: &PkgCacheFile, file: &IndexFile) -> bool;

		/// # Safety
		///
		/// If the Internal Pkg Pointer is NULL, operations can segfault
		unsafe fn u_find_pkg(self: &PkgCacheFile, name: String) -> UniquePtr<PkgIterator>;

		/// # Safety
		///
		/// If the Internal Pkg Pointer is NULL, operations can segfault
		unsafe fn u_begin(self: &PkgCacheFile) -> UniquePtr<PkgIterator>;
	}
}

impl raw::PkgCacheFile {
	/// Create the CacheFile.
	///
	/// It is advised to init the config and system before creating the
	/// cache. These bindings can be found in config::raw.
	pub fn new(volatile_files: &[String]) -> Result<UniquePtr<raw::PkgCacheFile>, AptErrors> {
		Ok(raw::u_create_cache(volatile_files)?)
	}

	/// Update the package lists, handle errors and return a Result.
	pub fn update(&self, progress: &mut raw::DynAcquireProgress) -> Result<(), AptErrors> {
		Ok(self.u_update(progress)?)
	}

	/// Return a package by name and optionally architecture.
	pub fn find_pkg(&self, name: &str) -> Option<UniquePtr<RawPackage>> {
		unsafe { self.u_find_pkg(name.to_string()).make_safe() }
	}

	/// Return the pointer to the start of the PkgIterator.
	pub fn begin(&self) -> Result<UniquePtr<RawPackage>, AptError> {
		unsafe {
			self.u_begin().make_safe().ok_or(AptError {
				is_error: true,
				msg: "No Packages Found!".to_string(),
			})
		}
	}
}
