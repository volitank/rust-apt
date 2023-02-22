use super::package::{RawPackage, RawVersion};

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {
	pub struct DepCache {
		ptr: UniquePtr<PkgDepCache>,
	}

	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/types.h");
		include!("rust-apt/apt-pkg-c/package.h");
		include!("rust-apt/apt-pkg-c/util.h");
		include!("rust-apt/apt-pkg-c/progress.h");
		include!("rust-apt/apt-pkg-c/depcache.h");

		type PkgDepCache;
		type Package = crate::raw::package::raw::Package;
		type Version = crate::raw::package::raw::Version;
		type DynOperationProgress = crate::raw::progress::raw::DynOperationProgress;

		pub fn init(self: &DepCache, callback: &mut DynOperationProgress) -> Result<()>;

		/// Autoinstall every broken package and run the problem resolver
		/// Returns false if the problem resolver fails.
		pub fn fix_broken(self: &DepCache) -> bool;

		/// Perform a Full Upgrade.
		/// Remove and install new packages if necessary.
		pub fn full_upgrade(self: &DepCache, progress: &mut DynOperationProgress) -> Result<()>;

		/// Perform a Safe Upgrade. Neither remove or install new packages.
		pub fn safe_upgrade(self: &DepCache, progress: &mut DynOperationProgress) -> Result<()>;

		/// Perform an Install Upgrade.
		/// New packages will be installed but nothing will be removed.
		pub fn install_upgrade(self: &DepCache, progress: &mut DynOperationProgress) -> Result<()>;

		/// Check if the package is upgradable.
		///
		/// ## skip_depcache:
		///
		/// Skipping the DepCache is unnecessary if it's already been
		/// initialized. If you're unsure use `false`
		///
		///   * [true] = Increases performance by skipping the pkgDepCache.
		///   * [false] = Use DepCache to check if the package is upgradable
		pub fn is_upgradable(self: &DepCache, pkg: &Package) -> bool;

		/// Is the Package auto installed? Packages marked as auto installed are
		/// usually dependencies.
		pub fn is_auto_installed(self: &DepCache, pkg: &Package) -> bool;

		/// Is the Package able to be auto removed?
		pub fn is_garbage(self: &DepCache, pkg: &Package) -> bool;

		/// Is the Package marked for install?
		pub fn marked_install(self: &DepCache, pkg: &Package) -> bool;

		/// Is the Package marked for upgrade?
		pub fn marked_upgrade(self: &DepCache, pkg: &Package) -> bool;

		/// Is the Package marked to be purged?
		pub fn marked_purge(self: &DepCache, pkg: &Package) -> bool;

		/// Is the Package marked for removal?
		pub fn marked_delete(self: &DepCache, pkg: &Package) -> bool;

		/// Is the Package marked for keep?
		pub fn marked_keep(self: &DepCache, pkg: &Package) -> bool;

		/// Is the Package marked for downgrade?
		pub fn marked_downgrade(self: &DepCache, pkg: &Package) -> bool;

		/// Is the Package marked for reinstall?
		pub fn marked_reinstall(self: &DepCache, pkg: &Package) -> bool;

		/// # Mark a package as automatically installed.
		///
		/// ## mark_auto:
		///   * [true] = Mark the package as automatically installed.
		///   * [false] = Mark the package as manually installed.
		pub fn mark_auto(self: &DepCache, pkg: &Package, mark_auto: bool);

		/// # Mark a package for keep.
		///
		/// ## Returns:
		///   * [true] if the mark was successful
		///   * [false] if the mark was unsuccessful
		///
		/// This means that the package will not be changed from its current
		/// version. This will not stop a reinstall, but will stop removal,
		/// upgrades and downgrades
		///
		/// We don't believe that there is any reason to unmark packages for
		/// keep. If someone has a reason, and would like it implemented, please
		/// put in a feature request.
		pub fn mark_keep(self: &DepCache, pkg: &Package) -> bool;

		/// # Mark a package for removal.
		///
		/// ## Returns:
		///   * [true] if the mark was successful
		///   * [false] if the mark was unsuccessful
		///
		/// ## purge:
		///   * [true] = Configuration files will be removed along with the
		///     package.
		///   * [false] = Only the package will be removed.
		pub fn mark_delete(self: &DepCache, pkg: &Package, purge: bool) -> bool;

		/// # Mark a package for installation.
		///
		/// ## auto_inst:
		///   * [true] = Additionally mark the dependencies for this package.
		///   * [false] = Mark only this package.
		///
		/// ## from_user:
		///   * [true] = The package will be marked manually installed.
		///   * [false] = The package will be unmarked automatically installed.
		///
		/// ## Returns:
		///   * [true] if the mark was successful
		///   * [false] if the mark was unsuccessful
		///
		/// If a package is already installed, at the latest version,
		/// and you mark that package for install you will get true,
		/// but the package will not be altered.
		/// `pkg.marked_install()` will be false
		pub fn mark_install(
			self: &DepCache,
			pkg: &Package,
			auto_inst: bool,
			from_user: bool,
		) -> bool;

		/// Set a version to be the candidate of it's package.
		pub fn set_candidate_version(self: &DepCache, ver: &Version);

		/// Get a pointer to the version that is set to be installed.
		///
		/// Safety: If there is no candidate the inner pointer will be null.
		/// This will cause segfaults if methods are used on a Null Version.
		pub fn unsafe_candidate_version(self: &DepCache, pkg: &Package) -> Version;

		/// # Mark a package for reinstallation.
		///
		/// ## Returns:
		///   * [true] if the mark was successful
		///   * [false] if the mark was unsuccessful
		///
		/// ## reinstall:
		///   * [true] = The package will be marked for reinstall.
		///   * [false] = The package will be unmarked for reinstall.
		pub fn mark_reinstall(self: &DepCache, pkg: &Package, reinstall: bool);

		/// Is the installed Package broken?
		pub fn is_now_broken(self: &DepCache, pkg: &Package) -> bool;

		/// Is the Package to be installed broken?
		pub fn is_inst_broken(self: &DepCache, pkg: &Package) -> bool;

		/// The number of packages marked for installation.
		pub fn install_count(self: &DepCache) -> u32;

		/// The number of packages marked for removal.
		pub fn delete_count(self: &DepCache) -> u32;

		/// The number of packages marked for keep.
		pub fn keep_count(self: &DepCache) -> u32;

		/// The number of packages with broken dependencies in the cache.
		pub fn broken_count(self: &DepCache) -> u32;

		/// The size of all packages to be downloaded.
		pub fn download_size(self: &DepCache) -> u64;

		/// The amount of space required for installing/removing the packages,"
		///
		/// i.e. the Installed-Size of all packages marked for installation"
		/// minus the Installed-Size of all packages for removal."
		pub fn disk_size(self: &DepCache) -> i64;
	}
}

impl raw::DepCache {
	pub fn candidate_version(&self, pkg: &RawPackage) -> Option<RawVersion> {
		let ptr = self.unsafe_candidate_version(pkg);

		match ptr.end() {
			true => None,
			false => Some(ptr),
		}
	}
}
