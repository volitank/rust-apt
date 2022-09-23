//! Contains Cache related structs.
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use cxx::UniquePtr;

use crate::config::init_config_system;
use crate::depcache::DepCache;
use crate::package::Package;
use crate::pkgmanager::PackageManager;
use crate::progress::{AcquireProgress, InstallProgress, OperationProgress};
use crate::records::Records;
use crate::resolver::ProblemResolver;
use crate::util::{apt_lock, apt_unlock, apt_unlock_inner, DiskSpace, Exception};
use crate::{depcache, package};

/// Struct for sorting packages.
pub type PackageSort = raw::PackageSort;
/// Enum for the Package Sorter.
pub type Sort = raw::Sort;
/// Enum to determine the upgrade type.
pub type Upgrade = depcache::raw::Upgrade;

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

/// Internal struct for managing references to pointers
#[derive(Debug)]
pub(crate) struct PointerMap {
	package_map: HashMap<String, Rc<RefCell<raw::PackagePtr>>>,
	version_map: HashMap<u32, Rc<RefCell<raw::VersionPtr>>>,
}

impl PointerMap {
	pub fn new() -> PointerMap {
		PointerMap {
			package_map: HashMap::new(),
			version_map: HashMap::new(),
		}
	}

	/// Remap all pointers after clearing the entire cache
	pub fn remap(&mut self, cache: &UniquePtr<raw::PkgCacheFile>) {
		// Remap packages to coincide with the new cache
		for (name, pkg_ptr) in self.package_map.iter_mut() {
			pkg_ptr.replace(
				raw::pkg_cache_find_name(cache, name.to_owned())
					// I think it's okay to panic here in the event of a null ptr
					.expect("Null package pointer found in pointer map"),
			);

			// Remap versions
			for ver_ptr in raw::pkg_version_list(&pkg_ptr.borrow()) {
				let ver_id = crate::package::raw::ver_id(&ver_ptr);

				// If the ID is in the map, replace it. Otherwise it doesn't need updating
				if let Some(ver) = self.version_map.get_mut(&ver_id) {
					ver.replace(ver_ptr);
				}
			}
		}
		// Throw away any of the pointers that are null
		// Really this says to keep it if it's not null
		self.package_map
			.retain(|_, pkg| !pkg.borrow().ptr.is_null())
	}

	/// Get a reference to a package pointer.
	/// Create it first if it doesn't exist.
	pub fn get_package(&mut self, pkg_ptr: raw::PackagePtr) -> Rc<RefCell<raw::PackagePtr>> {
		let pkg_name = crate::package::raw::get_fullname(&pkg_ptr, false);

		match self.package_map.get(&pkg_name) {
			// Package already exists, hand out a reference
			Some(pkg) => Rc::clone(pkg),
			// Package doesn't exist,
			// insert it into the map and then return a reference
			None => {
				let pkg = Rc::new(RefCell::new(pkg_ptr));
				let clone = Rc::clone(&pkg);
				// Insert the package into our map
				self.package_map.insert(pkg_name.to_owned(), pkg);
				// Return the reference cell
				clone
			},
		}
	}

	/// Get a reference to a version pointer.
	/// Create it first if it doesn't exist.
	pub fn get_version(&mut self, ver_ptr: raw::VersionPtr) -> Rc<RefCell<raw::VersionPtr>> {
		let ver_id = crate::package::raw::ver_id(&ver_ptr);

		match self.version_map.get(&ver_id) {
			// Version already exists, hand out a reference
			Some(ver) => Rc::clone(ver),
			// Version doesn't exist,
			// insert it into the map and then return a reference
			None => {
				let ver = Rc::new(RefCell::new(ver_ptr));
				let clone = Rc::clone(&ver);
				// Insert the version into our map
				self.version_map.insert(ver_id, ver);
				// Return the reference cell
				clone
			},
		}
	}
}

/// The main struct for accessing any and all `apt` data.
#[derive(Debug)]
pub struct Cache {
	pub ptr: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
	pub records: Rc<RefCell<Records>>,
	depcache: Rc<RefCell<DepCache>>,
	resolver: Rc<RefCell<ProblemResolver>>,
	pkgmanager: Rc<RefCell<PackageManager>>,
	pointer_map: Rc<RefCell<PointerMap>>,
	deb_pkglist: Vec<String>,
}

impl Default for Cache {
	fn default() -> Self { Self::new() }
}

impl Cache {
	fn internal_new(deb_pkgs: &[&str]) -> Result<Self, Exception> {
		init_config_system();
		let mut raw_deb_pkgs = vec![];
		for pkg in deb_pkgs {
			raw_deb_pkgs.push(pkg.to_string());
		}

		let cache_ptr = {
			match raw::pkg_cache_create(&raw_deb_pkgs) {
				Ok(ptr) => Rc::new(RefCell::new(ptr)),
				Err(err) => return Err(err),
			}
		};

		Ok(Self {
			records: Rc::new(RefCell::new(Records::new(Rc::clone(&cache_ptr)))),
			depcache: Rc::new(RefCell::new(DepCache::new(Rc::clone(&cache_ptr)))),
			resolver: Rc::new(RefCell::new(ProblemResolver::new(Rc::clone(&cache_ptr)))),
			pkgmanager: Rc::new(RefCell::new(PackageManager::new(Rc::clone(&cache_ptr)))),
			ptr: cache_ptr,
			pointer_map: Rc::new(RefCell::new(PointerMap::new())),
			deb_pkglist: raw_deb_pkgs,
		})
	}

	/// Initialize the configuration system, open and return the cache.
	/// This is the entry point for all operations of this crate.
	pub fn new() -> Self { Self::internal_new(&[]).unwrap() }

	/// The same thing as [`Cache::new`], but allows you to add local `.deb`
	/// files to the cache. This function returns an [`Exception`] if any of the
	/// `.deb` files cannot be found.
	pub fn debs(deb_files: &[&str]) -> Result<Self, Exception> { Self::internal_new(deb_files) }

	/// Clear the entire cache and start new.
	///
	/// This function would be used after `cache.update`
	/// Or after do_install if you plan on making more changes.
	///
	/// If you created the cache via [`Cache::debs`], this will return an
	/// [`Exception`] if the `.deb` files that were specified no longer exist on
	/// the system. If they do or you created the cache via [`Cache::new`] or
	/// [`Cache::default`], then this function will always return an [`Ok`], and
	/// it can safely be ran with `.unwrap`.
	pub fn clear(&self) -> Result<(), Exception> {
		let new_ptr = match raw::pkg_cache_create(&self.deb_pkglist) {
			Ok(ptr) => ptr,
			Err(err) => return Err(err),
		};

		// Replace all of the Cache references
		self.ptr.replace(new_ptr);
		self.records.replace(Records::new(Rc::clone(&self.ptr)));
		self.depcache.replace(DepCache::new(Rc::clone(&self.ptr)));
		self.resolver
			.replace(ProblemResolver::new(Rc::clone(&self.ptr)));
		self.pkgmanager
			.replace(PackageManager::new(Rc::clone(&self.ptr)));

		// Remap packages to coincide with the new cache
		self.pointer_map.borrow_mut().remap(&self.ptr.borrow());
		Ok(())
	}

	/// Updates the package cache and returns a Result
	///
	/// Here is an example of how you may parse the Error messages.
	///
	/// ```
	/// use rust_apt::cache::Cache;
	/// use rust_apt::progress::{AcquireProgress, AptAcquireProgress};
	///
	/// let cache = Cache::new();
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
	/// 
	/// # Known Errors:
	/// * E:Could not open lock file /var/lib/apt/lists/lock - open (13: Permission denied)
	/// * E:Unable to lock directory /var/lib/apt/lists/
	pub fn update(&self, progress: &mut Box<dyn AcquireProgress>) -> Result<(), Exception> {
		raw::cache_update(&self.ptr.borrow(), progress)
	}

	/// Mark all packages for upgrade
	///
	/// # Example:
	///
	/// ```
	/// use rust_apt::cache::{Cache, Upgrade};
	///
	/// let cache = Cache::new();
	///
	/// cache.upgrade(&Upgrade::FullUpgrade).unwrap();
	/// ```
	pub fn upgrade(&self, upgrade_type: &Upgrade) -> Result<(), Exception> {
		self.depcache
			.borrow()
			.upgrade(&mut NoOpProgress::new_box(), upgrade_type)
	}

	/// An iterator over the packages
	/// that will be altered when `cache.commit()` is called.
	///
	/// # sort_name:
	/// * [`true`] = Packages will be in alphabetical order
	/// * [`false`] = Packages will not be sorted by name
	pub fn get_changes(&self, sort_name: bool) -> impl Iterator<Item = Package> + '_ {
		let mut changed = Vec::new();
		let depcache = self.depcache.borrow();

		for pkg in raw::pkg_list(&self.ptr.borrow(), &PackageSort::default()) {
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
			changed.sort_by_cached_key(|pkg| package::raw::get_fullname(pkg, true));
		}

		changed
			.into_iter()
			.map(|pkg_ptr| self.make_package(pkg_ptr))
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
		self.resolver
			.borrow()
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
	/// use rust_apt::cache::Cache;
	/// use rust_apt::package::Mark;
	/// use rust_apt::progress::{AptAcquireProgress};
	///
	/// let cache = Cache::new();
	/// let pkg = cache.get("neovim").unwrap();
	/// let mut progress = AptAcquireProgress::new_box();
	///
	/// pkg.set(&Mark::Install).then_some(()).unwrap();
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
		self.pkgmanager
			.borrow()
			.get_archives(&mut self.records.borrow_mut(), progress)
	}

	/// Install, remove, and do any other actions requested by the cache.
	///
	/// # Returns:
	/// * A [`Result`] enum: the [`Ok`] variant if transaction was successful,
	///   and [`Err`] if there was an issue.
	///
	/// # Example:
	/// ```
	/// use rust_apt::cache::Cache;
	/// use rust_apt::package::Mark;
	/// use rust_apt::progress::{AptAcquireProgress, AptInstallProgress};
	///
	/// let cache = Cache::new();
	/// let pkg = cache.get("neovim").unwrap();
	/// let mut acquire_progress = AptAcquireProgress::new_box();
	/// let mut install_progress = AptInstallProgress::new_box();
	///
	/// pkg.set(&Mark::Install).then_some(()).unwrap();
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
	pub fn do_install(&self, progress: &mut Box<dyn InstallProgress>) -> Result<(), Exception> {
		self.pkgmanager.borrow().do_install(progress)
	}

	/// Handle get_archives and do_install in an easy wrapper.
	///
	/// # Returns:
	/// * A [`Result`]: the [`Ok`] variant if transaction was successful, and
	///   [`Err`] if there was an issue.
	/// # Example:
	/// ```
	/// use rust_apt::cache::Cache;
	/// use rust_apt::package::Mark;
	/// use rust_apt::progress::{AptAcquireProgress, AptInstallProgress};
	///
	/// let cache = Cache::new();
	/// let pkg = cache.get("neovim").unwrap();
	/// let mut acquire_progress = AptAcquireProgress::new_box();
	/// let mut install_progress = AptInstallProgress::new_box();
	///
	/// pkg.set(&Mark::Install).then_some(()).unwrap();
	/// pkg.protect();
	/// cache.resolve(true).unwrap();
	///
	/// // This needs root
	/// // cache.commit(&mut acquire_progress, &mut install_progress).unwrap();
	/// ```
	pub fn commit(
		&self,
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

	/// Clear any marked changes in the DepCache.
	pub fn clear_marked(&self) -> Result<(), Exception> {
		// Use our dummy OperationProgress struct.
		self.depcache.borrow().init(&mut NoOpProgress::new_box())
	}

	/// Returns an iterator of SourceURIs.
	///
	/// These are the files that `apt update` will fetch.
	pub fn sources(&self) -> impl Iterator<Item = raw::SourceFile> + '_ {
		raw::source_uris(&self.ptr.borrow()).into_iter()
	}

	/// Returns an iterator of Packages that provide the virtual package.
	///
	/// NOTE: This function is **ONLY** designed to get the list of packages of
	/// a virtual package. It also expects that you'll be installing the
	/// candidate version, and this likewise doesn't return a specific version
	/// to install. You probably want to use [`Package::rev_provides_list`]
	/// instead.
	pub fn provides(
		&self,
		virt_pkg: &Package,
		cand_only: bool,
	) -> impl Iterator<Item = Package> + '_ {
		raw::pkg_provides_list(&self.ptr.borrow(), &virt_pkg.ptr.borrow(), cand_only)
			.into_iter()
			.map(|pkg_ptr| self.make_package(pkg_ptr))
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

		// Match if the arch exists or not.
		match arch.is_empty() {
			true => {
				Some(self.make_package(
					raw::pkg_cache_find_name(&self.ptr.borrow(), name.to_owned()).ok()?,
				))
			},
			false => Some(
				self.make_package(
					raw::pkg_cache_find_name_arch(
						&self.ptr.borrow(),
						name.to_owned(),
						arch.to_owned(),
					)
					.ok()?,
				),
			),
		}
	}

	/// # Internal method to create a package from a pointer and deduplicate code.
	///
	/// If you don't use this on the cache struct you can pass the Cache as self
	///
	/// # Example:
	///
	/// We can't have a real example here as this deals with private fields
	///
	/// Say `pkg_ptr` is your package pointer from the cxx binding.
	///
	/// let pkg = Cache::make_package(&cache, pkg_ptr);
	///
	/// println!("{new_pkg}");
	pub(crate) fn make_package(&self, pkg_ptr: raw::PackagePtr) -> Package {
		Package::new(
			Rc::clone(&self.records),
			Rc::clone(&self.ptr),
			Rc::clone(&self.depcache),
			Rc::clone(&self.resolver),
			Rc::clone(&self.pointer_map),
			Rc::clone(&self.pointer_map.borrow_mut().get_package(pkg_ptr)),
		)
	}

	/// An iterator of packages in the cache.
	pub fn packages<'a>(&'a self, sort: &'a PackageSort) -> impl Iterator<Item = Package> + '_ {
		let mut pkg_list = raw::pkg_list(&self.ptr.borrow(), sort);
		if sort.names {
			pkg_list.sort_by_cached_key(|pkg| package::raw::get_fullname(pkg, true));
		}
		pkg_list
			.into_iter()
			.map(|pkg_ptr| self.make_package(pkg_ptr))
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

		type DynAcquireProgress = crate::progress::raw::DynAcquireProgress;

		include!("rust-apt/apt-pkg-c/cache.h");
		include!("rust-apt/apt-pkg-c/progress.h");
		include!("rust-apt/apt-pkg-c/records.h");

		// Main Initializers for apt:

		/// Create the CacheFile.
		///
		/// It is advised to init the config and system before creating the
		/// cache. These bindings can be found in config::raw.
		// TODO: Maybe this should return result. I believe this can fail with an apt error
		pub fn pkg_cache_create(deb_files: &[String]) -> Result<UniquePtr<PkgCacheFile>>;

		/// Update the package lists, handle errors and return a Result.
		// TODO: What kind of errors can be returned here?
		// TODO: Implement custom errors to match with apt errors
		pub fn cache_update(
			cache: &UniquePtr<PkgCacheFile>,
			progress: &mut DynAcquireProgress,
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

		/// Return a package by name.
		/// Ptr will be NULL if the package doesn't exist.
		// TODO: This should probably return result with an error
		// TODO: "Package does not exist"
		pub fn pkg_cache_find_name(
			cache: &UniquePtr<PkgCacheFile>,
			name: String,
		) -> Result<PackagePtr>;

		/// Return a package by name and architecture.
		/// Ptr will be NULL if the package doesn't exist.
		// TODO: This should probably return result with an error
		// TODO: "Package does not exist"
		pub fn pkg_cache_find_name_arch(
			cache: &UniquePtr<PkgCacheFile>,
			name: String,
			arch: String,
		) -> Result<PackagePtr>;

		// PackageFile Functions:

		/// The path to the PackageFile
		///
		/// Error "Unknown" if the information doesn't exist.
		pub fn filename(pkg_file: &PackageFile) -> Result<String>;

		/// The Archive of the PackageFile. ex: unstable
		///
		/// Error "Unknown" if the information doesn't exist.
		pub fn archive(pkg_file: &PackageFile) -> Result<String>;

		/// The Origin of the PackageFile. ex: Debian
		///
		/// Error "Unknown" if the information doesn't exist.
		pub fn origin(pkg_file: &PackageFile) -> Result<String>;

		/// The Codename of the PackageFile. ex: main, non-free
		///
		/// Error "Unknown" if the information doesn't exist.
		pub fn codename(pkg_file: &PackageFile) -> Result<String>;

		/// The Label of the PackageFile. ex: Debian
		///
		/// Error "Unknown" if the information doesn't exist.
		pub fn label(pkg_file: &PackageFile) -> Result<String>;

		/// The Hostname of the PackageFile. ex: deb.debian.org
		///
		/// Error "Unknown" if the information doesn't exist.
		pub fn site(pkg_file: &PackageFile) -> Result<String>;

		/// The Component of the PackageFile. ex: sid
		///
		/// Error "Unknown" if the information doesn't exist.
		pub fn component(pkg_file: &PackageFile) -> Result<String>;

		/// The Architecture of the PackageFile. ex: amd64
		///
		/// Error "Unknown" if the information doesn't exist.
		pub fn arch(pkg_file: &PackageFile) -> Result<String>;

		/// The Index Type of the PackageFile. Known values are:
		///
		/// Debian Package Index,
		/// Debian Translation Index,
		/// Debian dpkg status file,
		///
		/// Error "Unknown" if the information doesn't exist.
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
