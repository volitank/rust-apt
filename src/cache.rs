//! Contains Cache related structs.

use std::ops::Deref;

use cxx::{Exception, UniquePtr};
use once_cell::unsync::OnceCell;

use crate::config::init_config_system;
use crate::depcache::DepCache;
use crate::package::Package;
use crate::raw::cache::raw;
use crate::raw::package::RawPackage;
use crate::raw::pkgmanager::raw::{
	create_pkgmanager, create_problem_resolver, PackageManager, ProblemResolver,
};
use crate::raw::progress::{AcquireProgress, InstallProgress, OperationProgress};
use crate::raw::records::raw::Records;
use crate::util::{apt_lock, apt_unlock, apt_unlock_inner};

type RawRecords = UniquePtr<Records>;
type RawPkgManager = UniquePtr<PackageManager>;
type RawProblemResolver = UniquePtr<ProblemResolver>;

/// Internal struct to pass into [`self::Cache::resolve`]. The C++ library for
/// this wants a progress parameter for this, but it doesn't appear to be doing
/// anything. Furthermore, [the Python-APT implementation doesn't accept a
/// parameter for their dependency resolution funcionality](https://apt-team.pages.debian.net/python-apt/library/apt_pkg.html#apt_pkg.ProblemResolver.resolve),
/// so we should be safe to remove it here.
struct NoOpProgress {}

impl NoOpProgress {
	/// Return the AptAcquireProgress in a box
	/// To easily pass through for progress
	pub fn new_box() -> Box<dyn OperationProgress> { Box::new(NoOpProgress {}) }
}

impl OperationProgress for NoOpProgress {
	fn update(&mut self, _: String, _: f32) {}

	fn done(&mut self) {}
}

/// Selection of Upgrade type
pub enum Upgrade {
	/// Upgrade will Install new and Remove packages in addition to
	/// upgrading them.
	///
	/// Equivalent to `apt full-upgrade` and `apt-get dist-upgrade`.
	FullUpgrade,
	/// Upgrade will Not Install new or Remove packages.
	///
	/// Equivalent to `apt-get upgrade`.
	SafeUpgrade,
	/// Upgrade will Install new but not Remove packages.
	///
	/// Equivalent to `apt upgrade`.
	Upgrade,
}

pub enum Sort {
	/// Disable the sort method.
	Disable,
	/// Enable the sort method.
	Enable,
	/// Reverse the sort method.
	Reverse,
}

pub struct PackageSort {
	pub names: bool,
	pub upgradable: Sort,
	pub virtual_pkgs: Sort,
	pub installed: Sort,
	pub auto_installed: Sort,
	pub auto_removable: Sort,
}

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
pub struct Cache {
	cache: raw::Cache,
	depcache: OnceCell<DepCache>,
	records: OnceCell<RawRecords>,
	pkgmanager: OnceCell<RawPkgManager>,
	problem_resolver: OnceCell<RawProblemResolver>,
}

impl Cache {
	/// Initialize the configuration system, open and return the cache.
	/// This is the entry point for all operations of this crate.
	///
	/// deb_files allows you to add local `.deb` files to the cache.
	///
	/// This function returns an [`Exception`] if any of the `.deb` files cannot
	/// be found.
	///
	/// Note that if you run [`Cache::commit`] or [`Cache::update`],
	/// You will be required to make a new cache to perform any further changes
	pub fn new<T: ToString>(deb_files: &[T]) -> Result<Cache, Exception> {
		let deb_pkgs: Vec<_> = deb_files.iter().map(|d| d.to_string()).collect();

		init_config_system();
		Ok(Cache {
			cache: raw::create_cache(&deb_pkgs)?,
			depcache: OnceCell::new(),
			records: OnceCell::new(),
			pkgmanager: OnceCell::new(),
			problem_resolver: OnceCell::new(),
		})
	}

	/// Internal Method for generating the package list.
	pub fn raw_pkgs(&self) -> impl Iterator<Item = RawPackage> + '_ {
		self.begin().expect("Null PkgBegin!")
	}

	/// Get the DepCache
	pub fn depcache(&self) -> &DepCache {
		self.depcache
			.get_or_init(|| DepCache::new(self.create_depcache()))
	}

	/// Get the PkgRecords
	pub fn records(&self) -> &RawRecords { self.records.get_or_init(|| self.create_records()) }

	/// Get the PkgManager
	pub fn pkg_manager(&self) -> &RawPkgManager {
		self.pkgmanager
			.get_or_init(|| create_pkgmanager(&self.cache))
	}

	/// Get the ProblemResolver
	pub fn resolver(&self) -> &RawProblemResolver {
		self.problem_resolver
			.get_or_init(|| create_problem_resolver(&self.cache))
	}

	/// Iterate through the packages in a random order
	pub fn iter(&self) -> CacheIter {
		CacheIter {
			pkgs: self.begin().unwrap(),
			cache: self,
		}
	}

	/// An iterator of packages in the cache.
	pub fn packages(&self, sort: &PackageSort) -> impl Iterator<Item = Package> + '_ {
		let mut pkg_list = vec![];
		for pkg in self.raw_pkgs() {
			match sort.virtual_pkgs {
				// Virtual packages are enabled, include them.
				// This works differently than the rest. I should probably change defaults.
				Sort::Enable => {},
				// If disabled and pkg has no versions, exclude
				Sort::Disable => {
					if !pkg.has_versions() {
						continue;
					}
				},
				// If reverse and the package has versions, exclude
				// This section is for if you only want virtual packages
				Sort::Reverse => {
					if pkg.has_versions() {
						continue;
					}
				},
			}

			match sort.upgradable {
				// Virtual packages are enabled, include them.
				Sort::Disable => {},
				// If disabled and pkg has no versions, exclude
				Sort::Enable => {
					// TODO: These are probably wrong.
					// If the package isn't installed, then it can not be upgradable
					if !pkg.is_installed() || !self.depcache().is_upgradable(&pkg) {
						continue;
					}
				},
				// If reverse and the package has versions, exclude
				// This section is for if you only want virtual packages
				Sort::Reverse => {
					if pkg.is_installed() && self.depcache().is_upgradable(&pkg) {
						continue;
					}
				},
			}

			match sort.installed {
				// Installed Package is Disabled, so we keep them
				Sort::Disable => {},
				Sort::Enable => {
					if !pkg.is_installed() {
						continue;
					}
				},
				// Only include installed packages.
				Sort::Reverse => {
					if pkg.is_installed() {
						continue;
					}
				},
			}

			match sort.auto_installed {
				// Installed Package is Disabled, so we keep them
				Sort::Disable => {},
				Sort::Enable => {
					if !self.depcache().is_auto_installed(&pkg) {
						continue;
					}
				},
				// Only include installed packages.
				Sort::Reverse => {
					if self.depcache().is_auto_installed(&pkg) {
						continue;
					}
				},
			}

			match sort.auto_removable {
				// auto_removable is Disabled, so we keep them
				Sort::Disable => {},
				// If the package is not auto removable skip it.
				Sort::Enable => {
					// If the Package is installed or marked install then it cannot be Garbage.
					if (!pkg.is_installed() || !self.depcache().marked_install(&pkg))
						|| self.depcache().is_garbage(&pkg)
					{
						continue;
					}
				},
				// Only include installed packages.
				// If the package is auto removable skip it.
				Sort::Reverse => {
					if (pkg.is_installed() || self.depcache().marked_install(&pkg))
						&& self.depcache().is_garbage(&pkg)
					{
						continue;
					}
				},
			}

			// If this is reached we're clear to include the package.
			pkg_list.push(pkg);
		}

		if sort.names {
			pkg_list.sort_by_cached_key(|pkg| pkg.name().to_string());
		}

		pkg_list.into_iter().map(|pkg| Package::new(self, pkg))
	}

	/// Updates the package cache and returns a Result
	///
	/// Here is an example of how you may parse the Error messages.
	///
	/// ```
	/// use rust_apt::new_cache;
	/// use rust_apt::raw::progress::{AcquireProgress, AptAcquireProgress};
	///
	/// let cache = new_cache!().unwrap();
	/// let mut progress: Box<dyn AcquireProgress> = Box::new(AptAcquireProgress::new());

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
	/// # Known Errors:
	/// * E:Could not open lock file /var/lib/apt/lists/lock - open (13: Permission denied)
	/// * E:Unable to lock directory /var/lib/apt/lists/
	pub fn update(self, progress: &mut Box<dyn AcquireProgress>) -> Result<(), Exception> {
		self.cache.update(progress)?;
		Ok(())
	}

	/// Mark all packages for upgrade
	///
	/// # Example:
	///
	/// ```
	/// use rust_apt::new_cache;
	/// use rust_apt::cache::Upgrade;
	///
	/// let cache = new_cache!().unwrap();
	///
	/// cache.upgrade(&Upgrade::FullUpgrade).unwrap();
	/// ```
	pub fn upgrade(&self, upgrade_type: &Upgrade) -> Result<(), Exception> {
		let mut progress = NoOpProgress::new_box();
		match upgrade_type {
			Upgrade::FullUpgrade => self.depcache().full_upgrade(&mut progress),
			Upgrade::SafeUpgrade => self.depcache().safe_upgrade(&mut progress),
			Upgrade::Upgrade => self.depcache().install_upgrade(&mut progress),
		}
	}

	/// Resolve dependencies with the changes marked on all packages. This marks
	/// additional packages for installation/removal to satisfy the dependency
	/// chain.
	///
	/// Note that just running a `mark_*` function on a package doesn't
	/// guarantee that the selected state will be kept during dependency
	/// resolution. If you need such, make sure to run
	/// [`crate::package::Package::protect`] after marking your requested
	/// modifications.
	///
	/// If `fix_broken` is set to [`true`], the library will try to repair
	/// broken dependencies of installed packages.
	///
	/// Returns [`Err`] if there was an error reaching dependency resolution.
	#[allow(clippy::result_unit_err)]
	pub fn resolve(&self, fix_broken: bool) -> Result<(), Exception> {
		// Use our dummy OperationProgress struct. See
		// [`crate::cache::OperationProgress`] for why we need this.
		self.resolver()
			.resolve(fix_broken, &mut NoOpProgress::new_box())
	}

	/// Fetch any archives needed to complete the transaction.
	///
	/// # Returns:
	/// * A [`Result`] enum: the [`Ok`] variant if fetching succeeded, and
	///   [`Err`] if there was an issue.
	///
	/// # Example:
	/// ```
	/// use rust_apt::new_cache;
	/// use rust_apt::raw::progress::{AptAcquireProgress};
	///
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("neovim").unwrap();
	/// let mut progress = AptAcquireProgress::new_box();
	///
	/// pkg.mark_install(true, true);
	/// pkg.protect();
	/// cache.resolve(true).unwrap();
	///
	/// cache.get_archives(&mut progress).unwrap();
	/// ```
	/// # Known Errors:
	/// * W:Problem unlinking the file
	///   /var/cache/apt/archives/partial/neofetch_7.1.0-4_all.deb -
	///   PrepareFiles (13: Permission denied)
	/// * W:Problem unlinking the file
	///   /var/cache/apt/archives/partial/neofetch_7.1.0-4_all.deb -
	///   PrepareFiles (13: Permission denied)
	/// * W:Problem unlinking the file
	///   /var/cache/apt/archives/partial/neofetch_7.1.0-4_all.deb -
	///   PrepareFiles (13: Permission denied)
	/// * W:Problem unlinking the file
	///   /var/cache/apt/archives/partial/neofetch_7.1.0-4_all.deb -
	///   PrepareFiles (13: Permission denied)
	/// * W:Problem unlinking the file
	///   /var/cache/apt/archives/partial/neofetch_7.1.0-4_all.deb -
	///   PrepareFiles (13: Permission denied)
	/// * W:Problem unlinking the file /var/log/apt/eipp.log.xz - FileFd::Open
	///   (13: Permission denied)
	/// * W:Could not open file /var/log/apt/eipp.log.xz - open (17: File
	///   exists)
	/// * W:Could not open file '/var/log/apt/eipp.log.xz' - EIPP::OrderInstall
	///   (17: File exists)
	/// * E:Internal Error, ordering was unable to handle the media swap"
	pub fn get_archives(&self, progress: &mut Box<dyn AcquireProgress>) -> Result<(), Exception> {
		self.pkg_manager()
			.get_archives(&self.cache, self.records(), progress)
	}

	/// Install, remove, and do any other actions requested by the cache.
	///
	/// # Returns:
	/// * A [`Result`] enum: the [`Ok`] variant if transaction was successful,
	///   and [`Err`] if there was an issue.
	///
	/// # Example:
	/// ```
	/// use rust_apt::new_cache;
	/// use rust_apt::raw::progress::{AptAcquireProgress, AptInstallProgress};
	///
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("neovim").unwrap();
	/// let mut acquire_progress = AptAcquireProgress::new_box();
	/// let mut install_progress = AptInstallProgress::new_box();
	///
	/// pkg.mark_install(true, true);
	/// pkg.protect();
	/// cache.resolve(true).unwrap();
	///
	/// // These need root
	/// // cache.get_archives(&mut acquire_progress).unwrap();
	/// // cache.do_install(&mut install_progress).unwrap();
	/// ```
	///
	/// # Known Errors:
	/// * W:Problem unlinking the file /var/log/apt/eipp.log.xz - FileFd::Open
	///   (13: Permission denied)
	/// * W:Could not open file /var/log/apt/eipp.log.xz - open (17: File
	///   exists)
	/// * W:Could not open file '/var/log/apt/eipp.log.xz' - EIPP::OrderInstall
	///   (17: File exists)
	/// * E:Could not create temporary file for /var/lib/apt/extended_states -
	///   mkstemp (13: Permission denied)
	/// * E:Failed to write temporary StateFile /var/lib/apt/extended_states
	/// * W:Could not open file '/var/log/apt/term.log' - OpenLog (13:
	///   Permission denied)
	/// * E:Sub-process /usr/bin/dpkg returned an error code (2)
	/// * W:Problem unlinking the file /var/cache/apt/pkgcache.bin -
	///   pkgDPkgPM::Go (13: Permission denied)
	pub fn do_install(self, progress: &mut Box<dyn InstallProgress>) -> Result<(), Exception> {
		self.pkg_manager().do_install(progress)
	}

	/// Handle get_archives and do_install in an easy wrapper.
	///
	/// # Returns:
	/// * A [`Result`]: the [`Ok`] variant if transaction was successful, and
	///   [`Err`] if there was an issue.
	/// # Example:
	/// ```
	/// use rust_apt::new_cache;
	/// use rust_apt::raw::progress::{AptAcquireProgress, AptInstallProgress};
	///
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("neovim").unwrap();
	/// let mut acquire_progress = AptAcquireProgress::new_box();
	/// let mut install_progress = AptInstallProgress::new_box();
	///
	/// pkg.mark_install(true, true);
	/// pkg.protect();
	/// cache.resolve(true).unwrap();
	///
	/// // This needs root
	/// // cache.commit(&mut acquire_progress, &mut install_progress).unwrap();
	/// ```
	pub fn commit(
		self,
		progress: &mut Box<dyn AcquireProgress>,
		install_progress: &mut Box<dyn InstallProgress>,
	) -> Result<(), Exception> {
		// Lock the whole thing so as to prevent tamper
		apt_lock()?;

		// The archives can be grabbed during the apt lock.
		self.get_archives(progress)?;

		// If the system is locked we will want to unlock the dpkg files.
		// This way when dpkg is running it can access its files.
		apt_unlock_inner();

		// Perform the operation.
		self.do_install(install_progress)?;

		// Finally Unlock the whole thing.
		apt_unlock();
		Ok(())
	}

	/// Get a single package.
	///
	/// `cache.get("apt")` Returns a Package object for the native arch.
	///
	/// `cache.get("apt:i386")` Returns a Package object for the i386 arch
	pub fn get(&self, name: &str) -> Option<Package> {
		Some(Package::new(self, self.find_pkg(name)?))
	}

	/// An iterator over the packages
	/// that will be altered when `cache.commit()` is called.
	///
	/// # sort_name:
	/// * [`true`] = Packages will be in alphabetical order
	/// * [`false`] = Packages will not be sorted by name
	pub fn get_changes(&self, sort_name: bool) -> impl Iterator<Item = Package> + '_ {
		let mut changed = Vec::new();
		let depcache = self.depcache();

		for pkg in self.raw_pkgs() {
			if depcache.marked_install(&pkg)
				|| depcache.marked_delete(&pkg)
				|| depcache.marked_upgrade(&pkg)
				|| depcache.marked_downgrade(&pkg)
				|| depcache.marked_reinstall(&pkg)
			{
				changed.push(pkg);
			}
		}

		if sort_name {
			// Sort by cached key seems to be the fastest for what we're doing.
			// Maybe consider impl ord or something for these.
			changed.sort_by_cached_key(|pkg| pkg.name().to_string());
		}

		changed
			.into_iter()
			.map(|pkg_ptr| Package::new(self, pkg_ptr))
	}
}

/// Iterator Implementation for the Cache.
pub struct CacheIter<'a> {
	pkgs: RawPackage,
	cache: &'a Cache,
}

impl<'a> Iterator for CacheIter<'a> {
	type Item = Package<'a>;

	fn next(&mut self) -> Option<Self::Item> { Some(Package::new(self.cache, self.pkgs.next()?)) }
}

impl<'a> IntoIterator for &'a Cache {
	type IntoIter = CacheIter<'a>;
	type Item = Package<'a>;

	fn into_iter(self) -> Self::IntoIter { self.iter() }
}

/// Implementation to be able to call the raw cache methods from the high level
/// struct
impl Deref for Cache {
	type Target = raw::Cache;

	#[inline]
	fn deref(&self) -> &raw::Cache { &self.cache }
}
