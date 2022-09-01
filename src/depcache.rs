//! Contain DepCache related structs
use std::cell::RefCell;
use std::rc::Rc;

use cxx::UniquePtr;

use crate::package;
use crate::progress::OperationProgress;
use crate::util::{DiskSpace, Exception};

/// Internal Struct for managing the pkgDepCache.
#[derive(Debug)]
pub(crate) struct DepCache {
	cache: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
}

impl DepCache {
	pub(crate) fn new(cache: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>) -> Self {
		DepCache { cache }
	}

	/// Clear any marked changes in the DepCache.
	// TODO: What kind of errors can be returned here?
	// TODO: Implement custom errors to match with apt errors
	pub fn init(&self, op_progress: &mut Box<dyn OperationProgress>) -> Result<(), Exception> {
		raw::depcache_init(&self.cache.borrow(), op_progress)
	}

	pub fn upgrade(
		&self,
		op_progress: &mut Box<dyn OperationProgress>,
		upgrade_type: &raw::Upgrade,
	) -> Result<(), Exception> {
		raw::depcache_upgrade(&self.cache.borrow(), op_progress, upgrade_type)
	}

	/// The number of packages marked for installation.
	pub fn install_count(&self) -> u32 { raw::install_count(&self.cache.borrow()) }

	/// The number of packages marked for removal.
	pub fn delete_count(&self) -> u32 { raw::delete_count(&self.cache.borrow()) }

	/// The number of packages marked for keep.
	pub fn keep_count(&self) -> u32 { raw::keep_count(&self.cache.borrow()) }

	/// The number of packages with broken dependencies in the cache.
	pub fn broken_count(&self) -> u32 { raw::broken_count(&self.cache.borrow()) }

	/// The size of all packages to be downloaded.
	pub fn download_size(&self) -> u64 { raw::download_size(&self.cache.borrow()) }

	/// The amount of space required for installing/removing the packages,"
	///
	/// i.e. the Installed-Size of all packages marked for installation"
	/// minus the Installed-Size of all packages for removal."
	pub fn disk_size(&self) -> DiskSpace {
		let size = raw::disk_size(&self.cache.borrow());
		if size < 0 {
			return DiskSpace::Free(-size as u64);
		}
		DiskSpace::Require(size as u64)
	}

	pub fn is_upgradable(&self, pkg_ptr: &raw::PackagePtr, skip_depcache: bool) -> bool {
		raw::pkg_is_upgradable(&self.cache.borrow(), pkg_ptr, skip_depcache)
	}

	pub fn is_auto_installed(&self, pkg_ptr: &raw::PackagePtr) -> bool {
		raw::pkg_is_auto_installed(&self.cache.borrow(), pkg_ptr)
	}

	pub fn is_auto_removable(&self, pkg_ptr: &raw::PackagePtr) -> bool {
		(package::raw::pkg_is_installed(pkg_ptr)
			|| raw::pkg_marked_install(&self.cache.borrow(), pkg_ptr))
			&& raw::pkg_is_garbage(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_install(&self, pkg_ptr: &raw::PackagePtr) -> bool {
		raw::pkg_marked_install(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_upgrade(&self, pkg_ptr: &raw::PackagePtr) -> bool {
		raw::pkg_marked_upgrade(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_purge(&self, pkg_ptr: &raw::PackagePtr) -> bool {
		raw::pkg_marked_purge(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_delete(&self, pkg_ptr: &raw::PackagePtr) -> bool {
		raw::pkg_marked_delete(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_keep(&self, pkg_ptr: &raw::PackagePtr) -> bool {
		raw::pkg_marked_keep(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_downgrade(&self, pkg_ptr: &raw::PackagePtr) -> bool {
		raw::pkg_marked_downgrade(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_reinstall(&self, pkg_ptr: &raw::PackagePtr) -> bool {
		raw::pkg_marked_reinstall(&self.cache.borrow(), pkg_ptr)
	}

	pub fn mark_auto(&self, pkg_ptr: &raw::PackagePtr, mark_auto: bool) {
		raw::mark_auto(&self.cache.borrow(), pkg_ptr, mark_auto)
	}

	pub fn mark_keep(&self, pkg_ptr: &raw::PackagePtr) -> bool {
		raw::mark_keep(&self.cache.borrow(), pkg_ptr)
	}

	pub fn mark_delete(&self, pkg_ptr: &raw::PackagePtr, purge: bool) -> bool {
		raw::mark_delete(&self.cache.borrow(), pkg_ptr, purge)
	}

	/// Mark a package for installation.
	///
	/// auto_inst = Auto Install dependencies of the package.
	/// from_user = Mark the package as installed from the User.
	pub fn mark_install(
		&self,
		pkg_ptr: &raw::PackagePtr,
		auto_inst: bool,
		from_user: bool,
	) -> bool {
		raw::mark_install(&self.cache.borrow(), pkg_ptr, auto_inst, from_user)
	}

	pub fn mark_reinstall(&self, pkg_ptr: &raw::PackagePtr, reinstall: bool) {
		raw::mark_reinstall(&self.cache.borrow(), pkg_ptr, reinstall)
	}

	/// Set a version to be the candidate of it's package
	pub fn set_candidate(&self, ver_ptr: &raw::VersionPtr) {
		raw::set_candidate_version(&self.cache.borrow(), ver_ptr)
	}

	pub fn is_now_broken(&self, pkg_ptr: &raw::PackagePtr) -> bool {
		raw::pkg_is_now_broken(&self.cache.borrow(), pkg_ptr)
	}

	pub fn is_inst_broken(&self, pkg_ptr: &raw::PackagePtr) -> bool {
		raw::pkg_is_inst_broken(&self.cache.borrow(), pkg_ptr)
	}
}

/// This module contains the bindings and structs shared with c++
///
/// # Segfaults:
///   * If a package or version pointer is passed from a different cache, these
///     bindings may segfault or cause undefined behavior.
#[cxx::bridge]
pub mod raw {
	/// Enum to determine the upgrade type.
	#[repr(u8)]
	pub enum Upgrade {
		/// Upgrade will Install new and Remove packages in addition to
		/// upgrading them.
		///
		/// Equivalent to `apt full-upgrade` and `apt-get dist-upgrade`.
		FullUpgrade,
		/// Upgrade will Install new but not Remove packages.
		///
		/// Equivalent to `apt upgrade`.
		Upgrade,
		/// Upgrade will Not Install new or Remove packages.
		///
		/// Equivalent to `apt-get upgrade`.
		SafeUpgrade,
	}

	unsafe extern "C++" {
		type PkgDepCache;

		type PackagePtr = crate::cache::raw::PackagePtr;
		type VersionPtr = crate::cache::raw::VersionPtr;
		type PkgCacheFile = crate::cache::raw::PkgCacheFile;
		type DynOperationProgress = crate::progress::raw::DynOperationProgress;

		include!("rust-apt/apt-pkg-c/cache.h");
		include!("rust-apt/apt-pkg-c/depcache.h");

		/// Clear any marked changes in the DepCache.
		// TODO: What kind of errors can be returned here?
		// TODO: Implement custom errors to match with apt errors
		pub fn depcache_init(
			cache: &UniquePtr<PkgCacheFile>,
			op_progress: &mut DynOperationProgress,
		) -> Result<()>;

		/// Upgrade all packages
		pub fn depcache_upgrade(
			cache: &UniquePtr<PkgCacheFile>,
			op_progress: &mut DynOperationProgress,
			upgrade_type: &Upgrade,
		) -> Result<()>;

		/// Is the Package upgradable?
		///
		/// `skip_depcache = true` increases performance by skipping the
		/// pkgDepCache Skipping the depcache is very unnecessary if it's
		/// already been initialized If you're not sure, set `skip_depcache =
		/// false`
		pub fn pkg_is_upgradable(
			cache: &UniquePtr<PkgCacheFile>,
			iterator: &PackagePtr,
			skip_decache: bool,
		) -> bool;

		/// Is the Package auto installed? Packages marked as auto installed are
		/// usually dependencies.
		pub fn pkg_is_auto_installed(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr)
			-> bool;

		/// Is the Package able to be auto removed?
		pub fn pkg_is_garbage(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marked for install?
		pub fn pkg_marked_install(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marked for upgrade?
		pub fn pkg_marked_upgrade(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marrked to be purged?
		pub fn pkg_marked_purge(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marked for removal?
		pub fn pkg_marked_delete(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marked for keep?
		pub fn pkg_marked_keep(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marked for downgrade?
		pub fn pkg_marked_downgrade(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marked for reinstall?
		pub fn pkg_marked_reinstall(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Mark a package as automatically installed.
		pub fn mark_auto(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr, mark_auto: bool);

		/// Mark a package for keeping.
		pub fn mark_keep(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Mark a package for removal.
		pub fn mark_delete(
			cache: &UniquePtr<PkgCacheFile>,
			wrapper: &PackagePtr,
			purge: bool,
		) -> bool;

		/// Mark a package for installation.
		///
		/// auto_inst = Auto Install dependencies of the package.
		/// from_user = Mark the package as installed from the User.
		pub fn mark_install(
			cache: &UniquePtr<PkgCacheFile>,
			wrapper: &PackagePtr,
			auto_inst: bool,
			from_user: bool,
		) -> bool;

		/// Mark a package for reinstallation.
		pub fn mark_reinstall(
			cache: &UniquePtr<PkgCacheFile>,
			wrapper: &PackagePtr,
			reinstall: bool,
		);

		/// Set version to be the candidate of it's package.
		pub fn set_candidate_version(cache: &UniquePtr<PkgCacheFile>, wrapper: &VersionPtr);

		/// Is the installed Package broken?
		pub fn pkg_is_now_broken(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package to be installed broken?
		pub fn pkg_is_inst_broken(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// The number of packages marked for installation.
		pub fn install_count(cache: &UniquePtr<PkgCacheFile>) -> u32;

		/// The number of packages marked for removal.
		pub fn delete_count(cache: &UniquePtr<PkgCacheFile>) -> u32;

		/// The number of packages marked for keep.
		pub fn keep_count(cache: &UniquePtr<PkgCacheFile>) -> u32;

		/// The number of packages with broken dependencies in the cache.
		pub fn broken_count(cache: &UniquePtr<PkgCacheFile>) -> u32;

		/// The size of all packages to be downloaded.
		pub fn download_size(cache: &UniquePtr<PkgCacheFile>) -> u64;

		/// The amount of space required for installing/removing the packages,"
		///
		/// i.e. the Installed-Size of all packages marked for installation"
		/// minus the Installed-Size of all packages for removal."
		pub fn disk_size(cache: &UniquePtr<PkgCacheFile>) -> i64;
	}
}
