//! Contains Cache related structs.
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use cxx::{Exception, UniquePtr};

use crate::config::init_config_system;
use crate::depcache::DepCache;
use crate::package;
use crate::package::Package;
use crate::progress::UpdateProgress;
use crate::records::Records;
use crate::util::DiskSpace;

/// Struct for sorting packages.
pub type PackageSort = raw::PackageSort;
/// Enum for the Package Sorter.
pub type Sort = raw::Sort;

impl Default for PackageSort {
	fn default() -> PackageSort {
		PackageSort {
			names: false,
			upgradable: Sort::Disable,
			virtual_pkgs: Sort::Disable,
			installed: Sort::Disable,
			auto_installed: Sort::Disable,
			auto_removable: Sort::Disable,
		}
	}
}

impl PackageSort {
	/// Packages will be sorted by their names a -> z.
	pub fn names(mut self) -> Self {
		self.names = true;
		self
	}

	/// Only packages that are upgradable will be included.
	pub fn upgradable(mut self) -> Self {
		self.upgradable = Sort::Enable;
		self
	}

	/// Only packages that are NOT upgradable will be included.
	pub fn not_upgradable(mut self) -> Self {
		self.upgradable = Sort::Reverse;
		self
	}

	/// Virtual packages will be included.
	pub fn include_virtual(mut self) -> Self {
		self.virtual_pkgs = Sort::Enable;
		self
	}

	/// Only Virtual packages will be included.
	pub fn only_virtual(mut self) -> Self {
		self.virtual_pkgs = Sort::Reverse;
		self
	}

	/// Only packages that are installed will be included.
	pub fn installed(mut self) -> Self {
		self.installed = Sort::Enable;
		self
	}

	/// Only packages that are NOT installed will be included.
	pub fn not_installed(mut self) -> Self {
		self.installed = Sort::Reverse;
		self
	}

	/// Only packages that are auto installed will be included.
	pub fn auto_installed(mut self) -> Self {
		self.auto_installed = Sort::Enable;
		self
	}

	/// Only packages that are manually installed will be included.
	pub fn manually_installed(mut self) -> Self {
		self.auto_installed = Sort::Reverse;
		self
	}

	/// Only packages that are auto removable will be included.
	pub fn auto_removable(mut self) -> Self {
		self.auto_removable = Sort::Enable;
		self
	}

	/// Only packages that are NOT auto removable will be included.
	pub fn not_auto_removable(mut self) -> Self {
		self.auto_removable = Sort::Reverse;
		self
	}
}

/// The main struct for accessing any and all `apt` data.
#[derive(Debug)]
pub struct Cache {
	pub ptr: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
	pub records: Rc<RefCell<Records>>,
	depcache: Rc<RefCell<DepCache>>,
}

impl Default for Cache {
	fn default() -> Self { Self::new() }
}

impl Cache {
	/// Initialize the configuration system, open and return the cache.
	///
	/// This is the entry point for all operations of this crate.
	pub fn new() -> Self {
		init_config_system();
		let cache_ptr = Rc::new(RefCell::new(raw::pkg_cache_create()));
		Self {
			records: Rc::new(RefCell::new(Records::new(Rc::clone(&cache_ptr)))),
			depcache: Rc::new(RefCell::new(DepCache::new(Rc::clone(&cache_ptr)))),
			ptr: cache_ptr,
		}
	}

	/// Clears all changes made to packages.
	///
	/// Currently this doesn't do anything as we can't manipulate packages.
	pub fn clear(&self) { self.depcache.borrow().clear(); }

	/// Updates the package cache and returns a Result
	///
	/// Here is an example of how you may parse the Error messages.
	///
	/// ```
	/// use rust_apt::cache::Cache;
	/// use rust_apt::progress::{UpdateProgress, AptUpdateProgress};
	///
	/// let cache = Cache::new();
	/// let mut progress: Box<dyn UpdateProgress> = Box::new(AptUpdateProgress::new());

	/// if let Err(error) = cache.update(&mut progress) {
	///     for msg in error.what().split(';') {
	///         if msg.starts_with("E:") {
	///         println!("Error: {}", &msg[2..]);
	///         }
	///         if msg.starts_with("W:") {
	///             println!("Warning: {}", &msg[2..]);
	///         }
	///     }
	/// }
	/// ```
	pub fn update(&self, progress: &mut Box<dyn UpdateProgress>) -> Result<(), Exception> {
		raw::cache_update(&self.ptr.borrow(), progress)
	}

	/// Returns an iterator of SourceURIs.
	///
	/// These are the files that `apt update` will fetch.
	pub fn sources(&self) -> impl Iterator<Item = raw::SourceFile> + '_ {
		raw::source_uris(&self.ptr.borrow()).into_iter()
	}

	/// Returns an iterator of Packages that provide the virtual package
	pub fn provides(
		&self,
		virt_pkg: &Package,
		cand_only: bool,
	) -> impl Iterator<Item = Package> + '_ {
		raw::pkg_provides_list(&self.ptr.borrow(), &virt_pkg.ptr, cand_only)
			.into_iter()
			.map(|pkg| Package::new(Rc::clone(&self.records), Rc::clone(&self.depcache), pkg))
	}

	// Disabled as it doesn't really work yet. Would likely need to
	// Be on the objects them self and not the cache
	// pub fn validate(&self, ver: *mut raw::VerIterator) -> bool {
	// 	raw::validate(ver, self._cache)
	// }

	/// Get a single package.
	///
	/// `cache.get("apt")` Returns a Package object for the native arch.
	///
	/// `cache.get("apt:i386")` Returns a Package object for the i386 arch
	pub fn get<'a>(&'a self, name: &str) -> Option<Package<'a>> {
		let mut fields = name.split(':');

		let name = fields.next()?;
		let arch = fields.next().unwrap_or_default();
		let pkg_ptr = self.find_by_name(name, arch);

		if pkg_ptr.ptr.is_null() {
			return None;
		}
		Some(Package::new(
			Rc::clone(&self.records),
			Rc::clone(&self.depcache),
			pkg_ptr,
		))
	}

	/// Internal method for getting a package by name
	///
	/// Find a package by name and additionally architecture.
	///
	/// The returned iterator will either be at the end, or at a matching
	/// package.
	fn find_by_name(&self, name: &str, arch: &str) -> raw::PackagePtr {
		if !arch.is_empty() {
			return raw::pkg_cache_find_name_arch(
				&self.ptr.borrow(),
				name.to_owned(),
				arch.to_owned(),
			);
		}
		raw::pkg_cache_find_name(&self.ptr.borrow(), name.to_owned())
	}

	/// An iterator of packages in the cache.
	pub fn packages<'a>(&'a self, sort: &'a PackageSort) -> impl Iterator<Item = Package> + '_ {
		let mut pkg_list = raw::pkg_list(&self.ptr.borrow(), sort);
		if sort.names {
			pkg_list.sort_by_cached_key(|pkg| package::raw::get_fullname(pkg, true));
		}
		pkg_list
			.into_iter()
			.map(|pkg| Package::new(Rc::clone(&self.records), Rc::clone(&self.depcache), pkg))
	}

	/// The number of packages marked for installation.
	pub fn install_count(&self) -> u32 { self.depcache.borrow().install_count() }

	/// The number of packages marked for removal.
	pub fn delete_count(&self) -> u32 { self.depcache.borrow().delete_count() }

	/// The number of packages marked for keep.
	pub fn keep_count(&self) -> u32 { self.depcache.borrow().keep_count() }

	/// The number of packages with broken dependencies in the cache.
	pub fn broken_count(&self) -> u32 { self.depcache.borrow().broken_count() }

	/// The size of all packages to be downloaded.
	pub fn download_size(&self) -> u64 { self.depcache.borrow().download_size() }

	/// The amount of space required for installing/removing the packages,"
	///
	/// i.e. the Installed-Size of all packages marked for installation"
	/// minus the Installed-Size of all packages for removal."
	pub fn disk_size(&self) -> DiskSpace { self.depcache.borrow().disk_size() }
}

pub struct PackageFile {
	pkg_file: RefCell<raw::PackageFile>,
	pub cache: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
}

impl PackageFile {
	pub fn new(
		pkg_file: raw::PackageFile,
		cache: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
	) -> PackageFile {
		PackageFile {
			pkg_file: RefCell::new(pkg_file),
			cache,
		}
	}

	/// The path to the PackageFile
	pub fn filename(&self) -> Option<String> { raw::filename(&self.pkg_file.borrow()).ok() }

	/// The Archive of the PackageFile. ex: unstable
	pub fn archive(&self) -> Option<String> { raw::archive(&self.pkg_file.borrow()).ok() }

	/// The Origin of the PackageFile. ex: Debian
	pub fn origin(&self) -> Option<String> { raw::origin(&self.pkg_file.borrow()).ok() }

	/// The Codename of the PackageFile. ex: main, non-free
	pub fn codename(&self) -> Option<String> { raw::codename(&self.pkg_file.borrow()).ok() }

	/// The Label of the PackageFile. ex: Debian
	pub fn label(&self) -> Option<String> { raw::label(&self.pkg_file.borrow()).ok() }

	/// The Hostname of the PackageFile. ex: deb.debian.org
	pub fn site(&self) -> Option<String> { raw::site(&self.pkg_file.borrow()).ok() }

	/// The Component of the PackageFile. ex: sid
	pub fn component(&self) -> Option<String> { raw::component(&self.pkg_file.borrow()).ok() }

	/// The Architecture of the PackageFile. ex: amd64
	pub fn arch(&self) -> Option<String> { raw::arch(&self.pkg_file.borrow()).ok() }

	/// The Index Type of the PackageFile. Known values are:
	///
	/// Debian Package Index,
	/// Debian Translation Index,
	/// Debian dpkg status file,
	pub fn index_type(&self) -> Option<String> { raw::index_type(&self.pkg_file.borrow()).ok() }

	/// The Index of the PackageFile
	pub fn index(&self) -> u64 { raw::index(&self.pkg_file.borrow()) }

	/// Return true if the PackageFile is trusted.
	pub fn is_trusted(&self) -> bool {
		raw::pkg_file_is_trusted(&self.cache.borrow(), &mut self.pkg_file.borrow_mut())
	}
}

impl fmt::Debug for PackageFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"PackageFile <\n    Filename: {},\n    Archive: {},\n    Origin: {},\n    Codename: \
			 {},\n    Label: {},\n    Site: {},\n    Component: {},\n    Arch: {},\n    Index: \
			 {},\n    Index Type: {},\n    Trusted: {},\n>",
			self.filename().unwrap_or_else(|| String::from("Unknown")),
			self.archive().unwrap_or_else(|| String::from("Unknown")),
			self.origin().unwrap_or_else(|| String::from("Unknown")),
			self.codename().unwrap_or_else(|| String::from("Unknown")),
			self.label().unwrap_or_else(|| String::from("Unknown")),
			self.site().unwrap_or_else(|| String::from("Unknown")),
			self.component().unwrap_or_else(|| String::from("Unknown")),
			self.arch().unwrap_or_else(|| String::from("Unknown")),
			self.index(),
			self.index_type().unwrap_or_else(|| String::from("Unknown")),
			self.is_trusted(),
		)?;
		Ok(())
	}
}

impl fmt::Display for PackageFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{self:?}")?;
		Ok(())
	}
}

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {

	/// Struct representing a Source File.
	#[derive(Debug)]
	struct SourceFile {
		/// `http://deb.volian.org/volian/dists/scar/InRelease`
		uri: String,
		/// `deb.volian.org_volian_dists_scar_InRelease`
		filename: String,
	}

	/// A wrapper around the Apt pkgIterator.
	struct PackagePtr {
		ptr: UniquePtr<PkgIterator>,
	}

	/// A wrapper around the Apt verIterator.
	struct VersionPtr {
		ptr: UniquePtr<VerIterator>,
		desc: UniquePtr<DescIterator>,
	}

	/// A wrapper around PkgFileIterator.
	struct PackageFile {
		/// PackageFile UniquePtr.
		ptr: UniquePtr<PkgFile>,
	}

	/// A wrapper around VerFileIterator.
	struct VersionFile {
		/// VersionFile UniquePtr.
		ptr: UniquePtr<VerFileIterator>,
	}

	/// Enum to determine what will be sorted.
	#[derive(Debug)]
	pub enum Sort {
		/// Disable the sort method.
		Disable,
		/// Enable the sort method.
		Enable,
		/// Reverse the sort method.
		Reverse,
	}

	/// Struct for sorting packages.
	#[derive(Debug)]
	pub struct PackageSort {
		pub names: bool,
		pub upgradable: Sort,
		pub virtual_pkgs: Sort,
		pub installed: Sort,
		pub auto_installed: Sort,
		pub auto_removable: Sort,
	}

	unsafe extern "C++" {

		/// Apt C++ Type
		type PkgCacheFile;
		/// Apt C++ Type
		type PkgCache;
		/// Apt C++ Type
		type PkgSourceList;
		/// Apt C++ Type
		type PkgDepCache;

		/// Apt C++ Type
		type PkgIterator;
		/// Apt C++ Type
		type PkgFile;
		/// Apt C++ Type
		type VerIterator;
		/// Apt C++ Type
		type VerFileIterator;
		/// Apt C++ Type
		type DescIterator;

		type DynUpdateProgress = crate::progress::raw::DynUpdateProgress;

		include!("rust-apt/apt-pkg-c/cache.h");
		include!("rust-apt/apt-pkg-c/progress.h");
		include!("rust-apt/apt-pkg-c/records.h");

		// Main Initializers for apt:

		/// Create the CacheFile.
		///
		/// It is advised to init the config and system before creating the
		/// cache. These bindings can be found in config::raw.
		pub fn pkg_cache_create() -> UniquePtr<PkgCacheFile>;

		/// Update the package lists, handle errors and return a Result.
		pub fn cache_update(
			cache: &UniquePtr<PkgCacheFile>,
			progress: &mut DynUpdateProgress,
		) -> Result<()>;

		/// Get the package list uris. This is the files that are updated with
		/// `apt update`.
		pub fn source_uris(cache: &UniquePtr<PkgCacheFile>) -> Vec<SourceFile>;

		// Package Functions:

		/// Returns a Vector of all the packages in the cache.
		pub fn pkg_list(cache: &UniquePtr<PkgCacheFile>, sort: &PackageSort) -> Vec<PackagePtr>;

		// pkg_file_list and pkg_version_list should be in package::raw
		// I was unable to make this work so they remain here.

		/// Return a Vector of all the VersionFiles for a version.
		pub fn ver_file_list(ver: &VersionPtr) -> Vec<VersionFile>;

		/// Return a Vector of all the PackageFiles for a version.
		pub fn ver_pkg_file_list(ver: &VersionPtr) -> Vec<PackageFile>;

		/// Return a Vector of all the versions of a package.
		pub fn pkg_version_list(pkg: &PackagePtr) -> Vec<VersionPtr>;

		/// Return a Vector of all the packages that provide another. steam:i386
		/// provides steam.
		pub fn pkg_provides_list(
			cache: &UniquePtr<PkgCacheFile>,
			iterator: &PackagePtr,
			cand_only: bool,
		) -> Vec<PackagePtr>;

		/// Return a package by name. Ptr will be NULL if the package doesn't
		/// exist.
		pub fn pkg_cache_find_name(cache: &UniquePtr<PkgCacheFile>, name: String) -> PackagePtr;

		/// Return a package by name and architecture.
		/// Ptr will be NULL if the package doesn't exist.
		pub fn pkg_cache_find_name_arch(
			cache: &UniquePtr<PkgCacheFile>,
			name: String,
			arch: String,
		) -> PackagePtr;

		// PackageFile Functions:

		/// The path to the PackageFile
		pub fn filename(pkg_file: &PackageFile) -> Result<String>;

		/// The Archive of the PackageFile. ex: unstable
		pub fn archive(pkg_file: &PackageFile) -> Result<String>;

		/// The Origin of the PackageFile. ex: Debian
		pub fn origin(pkg_file: &PackageFile) -> Result<String>;

		/// The Codename of the PackageFile. ex: main, non-free
		pub fn codename(pkg_file: &PackageFile) -> Result<String>;

		/// The Label of the PackageFile. ex: Debian
		pub fn label(pkg_file: &PackageFile) -> Result<String>;

		/// The Hostname of the PackageFile. ex: deb.debian.org
		pub fn site(pkg_file: &PackageFile) -> Result<String>;

		/// The Component of the PackageFile. ex: sid
		pub fn component(pkg_file: &PackageFile) -> Result<String>;

		/// The Architecture of the PackageFile. ex: amd64
		pub fn arch(pkg_file: &PackageFile) -> Result<String>;

		/// The Index Type of the PackageFile. Known values are:
		///
		/// Debian Package Index,
		/// Debian Translation Index,
		/// Debian dpkg status file,
		pub fn index_type(pkg_file: &PackageFile) -> Result<String>;

		/// The Index of the PackageFile
		pub fn index(pkg_file: &PackageFile) -> u64;

		/// Return true if the PackageFile is trusted.
		pub fn pkg_file_is_trusted(
			cache: &UniquePtr<PkgCacheFile>,
			pkg_file: &mut PackageFile,
		) -> bool;
	}
}

impl fmt::Debug for raw::VersionPtr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"VersionPtr: {}:{}",
			package::raw::get_fullname(&package::raw::ver_parent(self), false),
			package::raw::ver_str(self)
		)?;
		Ok(())
	}
}

impl fmt::Debug for raw::PkgCacheFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "PkgCacheFile: {{ To Be Implemented }}")?;
		Ok(())
	}
}

impl fmt::Debug for raw::PkgDepCache {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "PkgDepCache: {{ To Be Implemented }}")?;
		Ok(())
	}
}

impl fmt::Debug for raw::PackagePtr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "PackagePtr: {}", package::raw::get_fullname(self, false))?;
		Ok(())
	}
}

impl fmt::Display for raw::SourceFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Source< Uri: {}, Filename: {}>", self.uri, self.filename)?;
		Ok(())
	}
}

impl fmt::Debug for raw::PackageFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "PackageFile: {{ To Be Implemented }}")?;
		Ok(())
	}
}

impl fmt::Debug for raw::VersionFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "VersionFile: {{ To Be Implemented }}")?;
		Ok(())
	}
}
