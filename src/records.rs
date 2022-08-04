use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use cxx::UniquePtr;

/// Internal Struct for managing package records.
#[derive(Debug)]
pub struct Records {
	pub(crate) ptr: raw::Records,
	pub(crate) cache: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
}

impl Records {
	pub fn new(cache: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>) -> Self {
		let record = raw::records_create(&cache.borrow());
		Records { ptr: record, cache }
	}

	pub fn lookup_desc(&mut self, desc: &UniquePtr<raw::DescIterator>) {
		raw::desc_file_lookup(&mut self.ptr, desc);
	}

	pub fn lookup_ver(&mut self, ver_file: &raw::VersionFile) {
		raw::ver_file_lookup(&mut self.ptr, ver_file);
	}

	pub fn description(&self) -> String { raw::long_desc(&self.ptr) }

	pub fn summary(&self) -> String { raw::short_desc(&self.ptr) }

	pub fn uri(&self, pkg_file: &raw::VersionFile) -> String {
		raw::ver_uri(&self.ptr, &self.cache.borrow(), pkg_file)
	}

	pub fn hash_find(&self, hash_type: &str) -> Option<String> {
		if let Ok(hash) = raw::hash_find(&self.ptr, hash_type.to_string()) {
			return Some(hash);
		}
		None
	}
}

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {
	/// A wrapper around the Apt pkgRecords class.
	struct Records {
		records: UniquePtr<PkgRecords>,
	}

	unsafe extern "C++" {
		type PkgRecords;

		type VersionFile = crate::cache::raw::VersionFile;
		type PackageFile = crate::cache::raw::PackageFile;
		type PkgCacheFile = crate::cache::raw::PkgCacheFile;
		type DescIterator = crate::cache::raw::DescIterator;

		include!("rust-apt/apt-pkg-c/cache.h");
		include!("rust-apt/apt-pkg-c/records.h");

		/// Package Record Management:

		/// Create the Package Records.
		pub fn records_create(cache: &UniquePtr<PkgCacheFile>) -> Records;

		/// Moves the Records into the correct place.
		pub fn ver_file_lookup(records: &mut Records, pkg_file: &VersionFile);

		/// Moves the Records into the correct place.
		pub fn desc_file_lookup(records: &mut Records, desc: &UniquePtr<DescIterator>);

		/// Return the URI for a version as determined by it's package file.
		/// A version could have multiple package files and multiple URIs.
		pub fn ver_uri(
			records: &Records,
			cache: &UniquePtr<PkgCacheFile>,
			ver_file: &VersionFile,
		) -> String;

		/// Return the translated long description of a Package.
		pub fn long_desc(records: &Records) -> String;

		/// Return the translated short description of a Package.
		pub fn short_desc(records: &Records) -> String;

		/// Find the hash of a Version. Returns "KeyError" (lul python) if there
		/// is no hash.
		pub fn hash_find(records: &Records, hash_type: String) -> Result<String>;
	}
}

impl fmt::Debug for raw::Records {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Records: {{ To Be Implemented }}")?;
		Ok(())
	}
}
