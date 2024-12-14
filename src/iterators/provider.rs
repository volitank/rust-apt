use std::fmt;

use cxx::UniquePtr;

use crate::raw::PrvIterator;
use crate::{Cache, Package, Version};

/// A Provider provides a Version and/or Package.
///
/// Typically if you had a virtual package you would get its providers
/// to find which Package/Version you should really install.
pub struct Provider<'a> {
	pub(crate) ptr: UniquePtr<PrvIterator>,
	cache: &'a Cache,
}

impl<'a> Provider<'a> {
	pub fn new(ptr: UniquePtr<PrvIterator>, cache: &'a Cache) -> Provider<'a> {
		Provider { ptr, cache }
	}

	/// Return the Target Package of the provider.
	pub fn package(&self) -> Package<'a> { Package::new(self.cache, unsafe { self.target_pkg() }) }

	/// Return the Target Version of the provider.
	pub fn version(&'a self) -> Version<'a> {
		Version::new(unsafe { self.target_ver() }, self.cache)
	}
}

impl fmt::Display for Provider<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let version = self.version();
		write!(
			f,
			"{} provides {} {}",
			self.name(),
			version.parent().fullname(false),
			version.version(),
		)?;
		Ok(())
	}
}

impl fmt::Debug for Provider<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Provider")
			.field("name", &self.name())
			.field("version", &self.version())
			.finish()
	}
}

#[cxx::bridge]
pub(crate) mod raw {
	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/package.h");

		type PrvIterator;

		type PkgIterator = crate::raw::PkgIterator;
		type VerIterator = crate::raw::VerIterator;

		/// The name of what this provider provides
		pub fn name(self: &PrvIterator) -> &str;

		/// The version string that this provides
		pub fn version_str(self: &PrvIterator) -> Result<&str>;

		/// The Target Package that can satisfy this provides
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn target_pkg(self: &PrvIterator) -> UniquePtr<PkgIterator>;

		/// The Target Version that can satisfy this provides
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn target_ver(self: &PrvIterator) -> UniquePtr<VerIterator>;

		#[cxx_name = "Index"]
		pub fn index(self: &PrvIterator) -> u64;
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
		unsafe fn unique(self: &PrvIterator) -> UniquePtr<PrvIterator>;
		pub fn raw_next(self: Pin<&mut PrvIterator>);
		pub fn end(self: &PrvIterator) -> bool;
	}
}
