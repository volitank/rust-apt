use std::cell::OnceCell;

use cxx::UniquePtr;

use crate::raw::{IndexFile, PkgFileIterator, VerFileIterator};
use crate::{Cache, PackageRecords};

/// Associates a version with a PackageFile
///
/// This allows a full description of all Versions in all files
pub struct VersionFile<'a> {
	pub(crate) ptr: UniquePtr<VerFileIterator>,
	cache: &'a Cache,
}

impl<'a> VersionFile<'a> {
	pub fn new(ptr: UniquePtr<VerFileIterator>, cache: &'a Cache) -> VersionFile<'a> {
		VersionFile { ptr, cache }
	}

	/// Return the PkgRecords Parser for the VersionFile
	pub fn lookup(&self) -> &PackageRecords { self.cache.records().ver_lookup(&self.ptr) }

	/// Return the PackageFile for this VersionFile
	pub fn package_file(&self) -> PackageFile<'a> {
		PackageFile::new(unsafe { self.ptr.package_file() }, self.cache)
	}
}

/// Stores information about the files used to generate the cache
///
/// Package files are referenced by Version structures to be able to know
/// after which Packages file includes this Version.
pub struct PackageFile<'a> {
	pub(crate) ptr: UniquePtr<PkgFileIterator>,
	cache: &'a Cache,
	index: OnceCell<UniquePtr<IndexFile>>,
}

impl<'a> PackageFile<'a> {
	pub fn new(ptr: UniquePtr<PkgFileIterator>, cache: &'a Cache) -> PackageFile<'a> {
		PackageFile {
			ptr,
			cache,
			index: OnceCell::new(),
		}
	}

	pub fn index_file(&self) -> &IndexFile {
		self.index
			.get_or_init(|| unsafe { self.cache.find_index(self) })
	}
}

cxx_convert_result!(
	PackageFile,
	/// The path to the PackageFile
	filename() -> &str,
	/// The Archive of the PackageFile. ex: unstable
	archive() -> &str,
	/// The Origin of the PackageFile. ex: Debian
	origin() -> &str,
	/// The Codename of the PackageFile. ex: main, non-free
	codename() -> &str,
	/// The Label of the PackageFile. ex: Debian
	label() -> &str,
	/// The Hostname of the PackageFile. ex: deb.debian.org
	site() -> &str,
	/// The Component of the PackageFile. ex: sid
	component() -> &str,
	/// The Architecture of the PackageFile. ex: amd64
	arch() -> &str,
	/// The Index Type of the PackageFile. Known values are:
	///
	/// Debian Package Index, Debian Translation Index
	/// and Debian dpkg status file,
	index_type() -> &str,
);

#[cxx::bridge]
pub(crate) mod raw {
	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/package.h");

		type VerFileIterator;
		type DescIterator;
		type PkgFileIterator;

		/// The path to the PackageFile
		pub fn filename(self: &PkgFileIterator) -> Result<&str>;

		/// The Archive of the PackageFile. ex: unstable
		pub fn archive(self: &PkgFileIterator) -> Result<&str>;

		/// The Origin of the PackageFile. ex: Debian
		pub fn origin(self: &PkgFileIterator) -> Result<&str>;

		/// The Codename of the PackageFile. ex: main, non-free
		pub fn codename(self: &PkgFileIterator) -> Result<&str>;

		/// The Label of the PackageFile. ex: Debian
		pub fn label(self: &PkgFileIterator) -> Result<&str>;

		/// The Hostname of the PackageFile. ex: deb.debian.org
		pub fn site(self: &PkgFileIterator) -> Result<&str>;

		/// The Component of the PackageFile. ex: sid
		pub fn component(self: &PkgFileIterator) -> Result<&str>;

		/// The Architecture of the PackageFile. ex: amd64
		pub fn arch(self: &PkgFileIterator) -> Result<&str>;

		/// The Index Type of the PackageFile. Known values are:
		///
		/// Debian Package Index, Debian Translation Index, Debian dpkg status
		/// file,
		pub fn index_type(self: &PkgFileIterator) -> Result<&str>;

		/// `true` if the PackageFile contains packages that can be downloaded
		pub fn is_downloadable(self: &PkgFileIterator) -> bool;

		/// The Index number of the PackageFile
		#[cxx_name = "Index"]
		pub fn index(self: &PkgFileIterator) -> u64;
		/// Clone the pointer.
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn unique(self: &PkgFileIterator) -> UniquePtr<PkgFileIterator>;
		pub fn raw_next(self: Pin<&mut PkgFileIterator>);
		pub fn end(self: &PkgFileIterator) -> bool;

		/// Return the package file associated with this version file.
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn package_file(self: &VerFileIterator) -> UniquePtr<PkgFileIterator>;

		#[cxx_name = "Index"]
		pub fn index(self: &VerFileIterator) -> u64;
		/// Clone the pointer.
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn unique(self: &VerFileIterator) -> UniquePtr<VerFileIterator>;
		pub fn raw_next(self: Pin<&mut VerFileIterator>);
		pub fn end(self: &VerFileIterator) -> bool;

		#[cxx_name = "Index"]
		pub fn index(self: &DescIterator) -> u64;
		/// Clone the pointer.
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn unique(self: &DescIterator) -> UniquePtr<DescIterator>;
		pub fn raw_next(self: Pin<&mut DescIterator>);
		pub fn end(self: &DescIterator) -> bool;
	}
}
