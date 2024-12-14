//! Contains Cache related structs.

use std::cell::OnceCell;
use std::fs;
use std::path::Path;

use cxx::{Exception, UniquePtr};

use crate::Package;
use crate::config::{Config, init_config_system};
use crate::depcache::DepCache;
use crate::error::{AptErrors, pending_error};
use crate::pkgmanager::raw::OrderResult;
use crate::progress::{AcquireProgress, InstallProgress, OperationProgress};
use crate::raw::{
	IntoRawIter, IterPkgIterator, PackageManager, PkgCacheFile, PkgIterator, ProblemResolver,
	create_cache, create_pkgmanager, create_problem_resolver,
};
use crate::records::{PackageRecords, SourceRecords};
use crate::util::{apt_lock, apt_unlock, apt_unlock_inner};

/// Selection of Upgrade type
#[repr(i32)]
#[derive(Clone, Debug)]
pub enum Upgrade {
	/// Upgrade will Install new and Remove packages in addition to
	/// upgrading them.
	///
	/// Equivalent to `apt full-upgrade` and `apt-get dist-upgrade`.
	FullUpgrade = 0,
	/// Upgrade will Install new but not Remove packages.
	///
	/// Equivalent to `apt upgrade`.
	Upgrade = 1,
	/// Upgrade will Not Install new or Remove packages.
	///
	/// Equivalent to `apt-get upgrade`.
	SafeUpgrade = 3,
}

/// Selection of how to sort
enum Sort {
	/// Disable the sort method.
	Disable,
	/// Enable the sort method.
	Enable,
	/// Reverse the sort method.
	Reverse,
}

/// Determines how to sort packages from the Cache.
pub struct PackageSort {
	names: bool,
	upgradable: Sort,
	virtual_pkgs: Sort,
	installed: Sort,
	auto_installed: Sort,
	auto_removable: Sort,
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
	pub(crate) ptr: UniquePtr<PkgCacheFile>,
	depcache: OnceCell<DepCache>,
	records: OnceCell<PackageRecords>,
	source_records: OnceCell<SourceRecords>,
	pkgmanager: OnceCell<UniquePtr<PackageManager>>,
	problem_resolver: OnceCell<UniquePtr<ProblemResolver>>,
	local_debs: Vec<String>,
}

impl Cache {
	/// Initialize the configuration system, open and return the cache.
	/// This is the entry point for all operations of this crate.
	///
	/// `local_files` allows you to temporarily add local files to the cache, as
	/// long as they are one of the following:
	///
	/// - `*.deb` or `*.ddeb` files
	/// - `Packages` and `Sources` files from apt repositories. These files can
	///   be compressed.
	/// - `*.dsc` or `*.changes` files
	/// - A valid directory containing the file `./debian/control`
	///
	/// This function returns an [`AptErrors`] if any of the files cannot
	/// be found or are invalid.
	///
	/// Note that if you run [`Cache::commit`] or [`Cache::update`],
	/// You will be required to make a new cache to perform any further changes
	pub fn new<T: AsRef<str>>(local_files: &[T]) -> Result<Cache, AptErrors> {
		let volatile_files: Vec<_> = local_files.iter().map(|d| d.as_ref()).collect();

		init_config_system();
		Ok(Cache {
			ptr: create_cache(&volatile_files)?,
			depcache: OnceCell::new(),
			records: OnceCell::new(),
			source_records: OnceCell::new(),
			pkgmanager: OnceCell::new(),
			problem_resolver: OnceCell::new(),
			local_debs: volatile_files
				.into_iter()
				.filter(|f| f.ends_with(".deb"))
				.map(|f| f.to_string())
				.collect(),
		})
	}

	/// Internal Method for generating the package list.
	pub fn raw_pkgs(&self) -> impl Iterator<Item = UniquePtr<PkgIterator>> {
		unsafe { self.begin().raw_iter() }
	}

	/// Get the DepCache
	pub fn depcache(&self) -> &DepCache {
		self.depcache
			.get_or_init(|| DepCache::new(unsafe { self.create_depcache() }))
	}

	/// Get the PkgRecords
	pub fn records(&self) -> &PackageRecords {
		self.records
			.get_or_init(|| PackageRecords::new(unsafe { self.create_records() }))
	}

	/// Get the PkgRecords
	pub fn source_records(&self) -> Result<&SourceRecords, AptErrors> {
		if let Some(records) = self.source_records.get() {
			return Ok(records);
		}

		match unsafe { self.ptr.source_records() } {
			Ok(raw_records) => {
				self.source_records
					.set(SourceRecords::new(raw_records))
					// Unwrap: This is verified empty at the beginning.
					.unwrap_or_default();
				// Unwrap: Records was just added above.
				Ok(self.source_records.get().unwrap())
			},
			Err(_) => Err(AptErrors::new()),
		}
	}

	/// Get the PkgManager
	pub fn pkg_manager(&self) -> &PackageManager {
		self.pkgmanager
			.get_or_init(|| unsafe { create_pkgmanager(self.depcache()) })
	}

	/// Get the ProblemResolver
	pub fn resolver(&self) -> &ProblemResolver {
		self.problem_resolver
			.get_or_init(|| unsafe { create_problem_resolver(self.depcache()) })
	}

	/// Iterate through the packages in a random order
	pub fn iter(&self) -> CacheIter {
		CacheIter {
			pkgs: unsafe { self.begin().raw_iter() },
			cache: self,
		}
	}

	/// An iterator of packages in the cache.
	pub fn packages(&self, sort: &PackageSort) -> impl Iterator<Item = Package> {
		let mut pkg_list = vec![];
		for pkg in self.raw_pkgs() {
			match sort.virtual_pkgs {
				// Virtual packages are enabled, include them.
				// This works differently than the rest. I should probably change defaults.
				Sort::Enable => {},
				// If disabled and pkg has no versions, exclude
				Sort::Disable => {
					if unsafe { pkg.versions().end() } {
						continue;
					}
				},
				// If reverse and the package has versions, exclude
				// This section is for if you only want virtual packages
				Sort::Reverse => {
					if unsafe { !pkg.versions().end() } {
						continue;
					}
				},
			}

			match sort.upgradable {
				// Virtual packages are enabled, include them.
				Sort::Disable => {},
				// If disabled and pkg has no versions, exclude
				Sort::Enable => {
					// If the package isn't installed, then it can not be upgradable
					if unsafe { pkg.current_version().end() }
						|| !self.depcache().is_upgradable(&pkg)
					{
						continue;
					}
				},
				// If reverse and the package is installed and upgradable, exclude
				// This section is for if you only want packages that are not upgradable
				Sort::Reverse => {
					if unsafe { !pkg.current_version().end() }
						&& self.depcache().is_upgradable(&pkg)
					{
						continue;
					}
				},
			}

			match sort.installed {
				// Installed Package is Disabled, so we keep them
				Sort::Disable => {},
				Sort::Enable => {
					if unsafe { pkg.current_version().end() } {
						continue;
					}
				},
				// Only include installed packages.
				Sort::Reverse => {
					if unsafe { !pkg.current_version().end() } {
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
					// If the Package isn't auto_removable skip
					if !self.depcache().is_garbage(&pkg) {
						continue;
					}
				},
				// If the package is auto removable skip it.
				Sort::Reverse => {
					if self.depcache().is_garbage(&pkg) {
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
	/// use rust_apt::progress::AcquireProgress;
	///
	/// let cache = new_cache!().unwrap();
	/// let mut progress = AcquireProgress::apt();
	/// if let Err(e) = cache.update(&mut progress) {
	///     for error in e.iter() {
	///         if error.is_error {
	///             println!("Error: {}", error.msg);
	///         } else {
	///             println!("Warning: {}", error.msg);
	///         }
	///     }
	/// }
	/// ```
	/// # Known Errors:
	/// * E:Could not open lock file /var/lib/apt/lists/lock - open (13:
	///   Permission denied)
	/// * E:Unable to lock directory /var/lib/apt/lists/
	pub fn update(self, progress: &mut AcquireProgress) -> Result<(), AptErrors> {
		Ok(self.ptr.update(progress.mut_status())?)
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
	/// cache.upgrade(Upgrade::FullUpgrade).unwrap();
	/// ```
	pub fn upgrade(&self, upgrade_type: Upgrade) -> Result<(), AptErrors> {
		let mut progress = OperationProgress::quiet();
		Ok(self
			.depcache()
			.upgrade(progress.pin().as_mut(), upgrade_type as i32)?)
	}

	/// Resolve dependencies with the changes marked on all packages. This marks
	/// additional packages for installation/removal to satisfy the dependency
	/// chain.
	///
	/// Note that just running a `mark_*` function on a package doesn't
	/// guarantee that the selected state will be kept during dependency
	/// resolution. If you need such, make sure to run
	/// [`crate::Package::protect`] after marking your requested
	/// modifications.
	///
	/// If `fix_broken` is set to [`true`], the library will try to repair
	/// broken dependencies of installed packages.
	///
	/// Returns [`Err`] if there was an error reaching dependency resolution.
	#[allow(clippy::result_unit_err)]
	pub fn resolve(&self, fix_broken: bool) -> Result<(), AptErrors> {
		Ok(self
			.resolver()
			.resolve(fix_broken, OperationProgress::quiet().pin().as_mut())?)
	}

	/// Autoinstall every broken package and run the problem resolver
	/// Returns false if the problem resolver fails.
	///
	/// # Example:
	///
	/// ```
	/// use rust_apt::new_cache;
	///
	/// let cache = new_cache!().unwrap();
	///
	/// cache.fix_broken();
	///
	/// for pkg in cache.get_changes(false) {
	///     println!("Pkg Name: {}", pkg.name())
	/// }
	/// ```
	pub fn fix_broken(&self) -> bool { self.depcache().fix_broken() }

	/// Fetch any archives needed to complete the transaction.
	///
	/// # Returns:
	/// * A [`Result`] enum: the [`Ok`] variant if fetching succeeded, and
	///   [`Err`] if there was an issue.
	///
	/// # Example:
	/// ```
	/// use rust_apt::new_cache;
	/// use rust_apt::progress::AcquireProgress;
	///
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("neovim").unwrap();
	/// let mut progress = AcquireProgress::apt();
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
	pub fn get_archives(&self, progress: &mut AcquireProgress) -> Result<(), Exception> {
		self.pkg_manager()
			.get_archives(&self.ptr, self.records(), progress.mut_status())
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
	/// use rust_apt::progress::{AcquireProgress, InstallProgress};
	///
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("neovim").unwrap();
	/// let mut acquire_progress = AcquireProgress::apt();
	/// let mut install_progress = InstallProgress::apt();
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
	pub fn do_install(self, progress: &mut InstallProgress) -> Result<(), AptErrors> {
		let res = match progress {
			InstallProgress::Fancy(inner) => self.pkg_manager().do_install(inner.pin().as_mut()),
			InstallProgress::Fd(fd) => self.pkg_manager().do_install_fd(*fd),
		};

		if pending_error() {
			return Err(AptErrors::new());
		}

		match res {
			OrderResult::Completed => {},
			OrderResult::Failed => panic!(
				"DoInstall failed with no error from libapt. Please report this as an issue."
			),
			OrderResult::Incomplete => {
				panic!("Result is 'Incomplete', please request media swapping as a feature.")
			},
			_ => unreachable!(),
		}

		Ok(())
	}

	/// Handle get_archives and do_install in an easy wrapper.
	///
	/// # Returns:
	/// * A [`Result`]: the [`Ok`] variant if transaction was successful, and
	///   [`Err`] if there was an issue.
	/// # Example:
	/// ```
	/// use rust_apt::new_cache;
	/// use rust_apt::progress::{AcquireProgress, InstallProgress};
	///
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("neovim").unwrap();
	/// let mut acquire_progress = AcquireProgress::apt();
	/// let mut install_progress = InstallProgress::apt();
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
		progress: &mut AcquireProgress,
		install_progress: &mut InstallProgress,
	) -> Result<(), AptErrors> {
		// Lock the whole thing so as to prevent tamper
		apt_lock()?;

		let config = Config::new();
		let archive_dir = config.dir("Dir::Cache::Archives", "/var/cache/apt/archives/");

		// Copy local debs into archives dir
		for deb in &self.local_debs {
			// If it reaches this point it really will be a valid filename, allegedly
			if let Some(filename) = Path::new(deb).file_name() {
				// Append the file name onto the archive dir
				fs::copy(deb, archive_dir.to_string() + &filename.to_string_lossy())?;
			}
		}

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
		Some(Package::new(self, unsafe {
			self.find_pkg(name).make_safe()?
		}))
	}

	/// An iterator over the packages
	/// that will be altered when `cache.commit()` is called.
	///
	/// # sort_name:
	/// * [`true`] = Packages will be in alphabetical order
	/// * [`false`] = Packages will not be sorted by name
	pub fn get_changes(&self, sort_name: bool) -> impl Iterator<Item = Package> {
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
	pkgs: IterPkgIterator,
	cache: &'a Cache,
}

impl<'a> Iterator for CacheIter<'a> {
	type Item = Package<'a>;

	fn next(&mut self) -> Option<Self::Item> { Some(Package::new(self.cache, self.pkgs.next()?)) }
}

#[cxx::bridge]
pub(crate) mod raw {
	impl UniquePtr<PkgRecords> {}

	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/cache.h");
		type PkgCacheFile;

		type PkgIterator = crate::raw::PkgIterator;
		type VerIterator = crate::raw::VerIterator;
		type PkgFileIterator = crate::raw::PkgFileIterator;
		type PkgRecords = crate::records::raw::PkgRecords;
		type SourceRecords = crate::records::raw::SourceRecords;
		type IndexFile = crate::records::raw::IndexFile;
		type PkgDepCache = crate::depcache::raw::PkgDepCache;
		type AcqTextStatus = crate::acquire::raw::AcqTextStatus;
		type PkgAcquire = crate::acquire::raw::PkgAcquire;

		/// Create the CacheFile.
		pub fn create_cache(volatile_files: &[&str]) -> Result<UniquePtr<PkgCacheFile>>;

		/// Update the package lists, handle errors and return a Result.
		pub fn update(self: &PkgCacheFile, progress: Pin<&mut AcqTextStatus>) -> Result<()>;

		/// Loads the index files into PkgAcquire.
		///
		/// Used to get to source list uris.
		///
		/// It's not clear if this returning a bool is useful.
		pub fn get_indexes(self: &PkgCacheFile, fetcher: &PkgAcquire) -> bool;

		/// Return a pointer to PkgDepcache.
		///
		/// # Safety
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn create_depcache(self: &PkgCacheFile) -> UniquePtr<PkgDepCache>;

		/// Return a pointer to PkgRecords.
		///
		/// # Safety
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn create_records(self: &PkgCacheFile) -> UniquePtr<PkgRecords>;

		unsafe fn source_records(self: &PkgCacheFile) -> Result<UniquePtr<SourceRecords>>;

		/// The priority of the Version as shown in `apt policy`.
		pub fn priority(self: &PkgCacheFile, version: &VerIterator) -> i32;

		/// Lookup the IndexFile of the Package file
		///
		/// # Safety
		///
		/// The IndexFile can not outlive PkgCacheFile.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn find_index(self: &PkgCacheFile, file: &PkgFileIterator) -> UniquePtr<IndexFile>;

		/// Return a package by name and optionally architecture.
		///
		/// # Safety
		///
		/// If the Internal Pkg Pointer is NULL, operations can segfault.
		/// You should call `make_safe()` asap to convert it to an Option.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn find_pkg(self: &PkgCacheFile, name: &str) -> UniquePtr<PkgIterator>;

		/// Return the pointer to the start of the PkgIterator.
		///
		/// # Safety
		///
		/// If the Internal Pkg Pointer is NULL, operations can segfault.
		/// You should call `raw_iter()` asap.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn begin(self: &PkgCacheFile) -> UniquePtr<PkgIterator>;
	}
}
