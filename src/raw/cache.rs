//! Contains Cache related structs.

use super::package::RawPackage;

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {

	pub struct Cache {
		ptr: UniquePtr<PkgCacheFile>,
	}

	impl UniquePtr<Records> {}

	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/types.h");
		include!("rust-apt/apt-pkg-c/package.h");
		include!("rust-apt/apt-pkg-c/util.h");
		include!("rust-apt/apt-pkg-c/depcache.h");
		include!("rust-apt/apt-pkg-c/records.h");
		include!("rust-apt/apt-pkg-c/progress.h");
		include!("rust-apt/apt-pkg-c/cache.h");
		type PkgCacheFile;

		type Package = crate::raw::package::raw::Package;
		type Version = crate::raw::package::raw::Version;
		type PackageFile = crate::raw::package::raw::PackageFile;
		type SourceURI = crate::raw::package::raw::SourceURI;

		type Records = crate::raw::records::raw::Records;
		type DepCache = crate::raw::depcache::raw::DepCache;

		type DynAcquireProgress = crate::raw::progress::raw::DynAcquireProgress;

		/// Create the CacheFile.
		///
		/// It is advised to init the config and system before creating the
		/// cache. These bindings can be found in config::raw.
		pub fn create_cache(deb_files: &[String]) -> Result<Cache>;

		// TODO: What kind of errors can be returned here?
		// TODO: Implement custom errors to match with apt errors
		/// Update the package lists, handle errors and return a Result.
		pub fn update(self: &Cache, progress: &mut DynAcquireProgress) -> Result<()>;

		/// Returns an iterator of SourceURIs.
		///
		/// These are the files that `apt update` will fetch.
		pub fn source_uris(self: &Cache) -> Vec<SourceURI>;

		pub fn create_depcache(self: &Cache) -> DepCache;

		pub fn create_records(self: &Cache) -> UniquePtr<Records>;

		/// The priority of the Version as shown in `apt policy`.
		pub fn priority(self: &Cache, version: &Version) -> i32;

		/// Lookup the IndexFile of the Package file
		pub fn find_index(self: &Cache, pkg_file: &mut PackageFile);

		/// Return true if the PackageFile is trusted.
		pub fn is_trusted(self: &Cache, pkg_file: &mut PackageFile) -> bool;

		/// Return a package by name and optionally architecture.
		pub fn unsafe_find_pkg(self: &Cache, name: String) -> Package;

		/// Return the pointer to the start of the PkgIterator.
		pub fn begin(self: &Cache) -> Result<Package>;

	}
}

impl raw::Cache {
	pub fn find_pkg(&self, name: &str) -> Option<RawPackage> {
		self.unsafe_find_pkg(name.to_string()).make_safe()
	}
}
