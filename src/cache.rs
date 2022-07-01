//! Contains Cache related structs.
use std::cell::RefCell;
use std::rc::Rc;

use cxx::{Exception, UniquePtr};

use crate::config::init_config_system;
use crate::package::Package;
use crate::progress::UpdateProgress;
use crate::raw::apt;

/// Struct for sorting packages.
pub type PackageSort = apt::PackageSort;
/// Enum for the Package Sorter.
pub type Sort = apt::Sort;

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

/// Internal Struct for managing package records.
#[derive(Debug)]
pub struct Records {
	pub(crate) ptr: apt::Records,
	pub(crate) cache: Rc<RefCell<UniquePtr<apt::PkgCacheFile>>>,
}

impl Records {
	pub fn new(cache: Rc<RefCell<UniquePtr<apt::PkgCacheFile>>>) -> Self {
		let record = apt::pkg_records_create(&cache.borrow());
		Records { ptr: record, cache }
	}

	pub fn lookup_desc(&mut self, desc: &UniquePtr<apt::DescIterator>) {
		apt::desc_file_lookup(&mut self.ptr, desc);
	}

	pub fn lookup_ver(&mut self, ver_file: &apt::PackageFile) {
		apt::ver_file_lookup(&mut self.ptr, ver_file);
	}

	pub fn description(&self) -> String { apt::long_desc(&self.ptr) }

	pub fn summary(&self) -> String { apt::short_desc(&self.ptr) }

	pub fn uri(&self, pkg_file: &apt::PackageFile) -> String {
		apt::ver_uri(&self.ptr, &self.cache.borrow(), pkg_file)
	}

	pub fn hash_find(&self, hash_type: &str) -> Option<String> {
		let hash = apt::hash_find(&self.ptr, hash_type.to_string());
		if hash == "KeyError" {
			return None;
		}
		Some(hash)
	}
}

/// Internal Struct for managing the pkgDepCache.
#[derive(Debug)]
pub struct DepCache {
	cache: Rc<RefCell<UniquePtr<apt::PkgCacheFile>>>,
}

impl DepCache {
	pub fn new(cache: Rc<RefCell<UniquePtr<apt::PkgCacheFile>>>) -> Self { DepCache { cache } }

	pub fn clear(&self) { apt::depcache_create(&self.cache.borrow()); }

	pub fn is_upgradable(&self, pkg_ptr: &apt::PackagePtr, skip_depcache: bool) -> bool {
		apt::pkg_is_upgradable(&self.cache.borrow(), pkg_ptr, skip_depcache)
	}

	pub fn is_auto_installed(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		apt::pkg_is_auto_installed(&self.cache.borrow(), pkg_ptr)
	}

	pub fn is_auto_removable(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		(apt::pkg_is_installed(pkg_ptr) || apt::pkg_marked_install(&self.cache.borrow(), pkg_ptr))
			&& apt::pkg_is_garbage(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_install(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		apt::pkg_marked_install(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_upgrade(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		apt::pkg_marked_upgrade(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_delete(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		apt::pkg_marked_delete(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_keep(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		apt::pkg_marked_keep(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_downgrade(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		apt::pkg_marked_downgrade(&self.cache.borrow(), pkg_ptr)
	}

	pub fn marked_reinstall(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		apt::pkg_marked_reinstall(&self.cache.borrow(), pkg_ptr)
	}

	pub fn is_now_broken(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		apt::pkg_is_now_broken(&self.cache.borrow(), pkg_ptr)
	}

	pub fn is_inst_broken(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		apt::pkg_is_inst_broken(&self.cache.borrow(), pkg_ptr)
	}
}

/// The main struct for accessing any and all `apt` data.
#[derive(Debug)]
pub struct Cache {
	pub ptr: Rc<RefCell<UniquePtr<apt::PkgCacheFile>>>,
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
		let cache_ptr = Rc::new(RefCell::new(apt::pkg_cache_create()));
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
		apt::cache_update(&self.ptr.borrow(), progress)
	}

	/// Returns an iterator of SourceURIs.
	///
	/// These are the files that `apt update` will fetch.
	pub fn sources(&self) -> impl Iterator<Item = apt::SourceFile> + '_ {
		apt::source_uris(&self.ptr.borrow()).into_iter()
	}

	/// Returns an iterator of Packages that provide the virtual package
	pub fn provides(
		&self,
		virt_pkg: &Package,
		cand_only: bool,
	) -> impl Iterator<Item = Package> + '_ {
		apt::pkg_provides_list(&self.ptr.borrow(), &virt_pkg.ptr, cand_only)
			.into_iter()
			.map(|pkg| Package::new(Rc::clone(&self.records), Rc::clone(&self.depcache), pkg))
	}

	// Disabled as it doesn't really work yet. Would likely need to
	// Be on the objects them self and not the cache
	// pub fn validate(&self, ver: *mut apt::VerIterator) -> bool {
	// 	apt::validate(ver, self._cache)
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
	fn find_by_name(&self, name: &str, arch: &str) -> apt::PackagePtr {
		if !arch.is_empty() {
			return apt::pkg_cache_find_name_arch(
				&self.ptr.borrow(),
				name.to_owned(),
				arch.to_owned(),
			);
		}
		apt::pkg_cache_find_name(&self.ptr.borrow(), name.to_owned())
	}

	/// An iterator of packages in the cache.
	pub fn packages<'a>(&'a self, sort: &'a PackageSort) -> impl Iterator<Item = Package> + '_ {
		let mut pkg_list = apt::pkg_list(&self.ptr.borrow(), sort);
		if sort.names {
			pkg_list.sort_by_cached_key(|pkg| apt::get_fullname(pkg, true));
		}
		pkg_list
			.into_iter()
			.map(|pkg| Package::new(Rc::clone(&self.records), Rc::clone(&self.depcache), pkg))
	}

	/// The number of packages marked for installation.
	pub fn install_count(&self) -> u32 { apt::install_count(&self.ptr.borrow()) }

	/// The number of packages marked for removal.
	pub fn delete_count(&self) -> u32 { apt::delete_count(&self.ptr.borrow()) }

	/// The number of packages marked for keep.
	pub fn keep_count(&self) -> u32 { apt::keep_count(&self.ptr.borrow()) }

	/// The number of packages with broken dependencies in the cache.
	pub fn broken_count(&self) -> u32 { apt::broken_count(&self.ptr.borrow()) }

	/// The size of all packages to be downloaded.
	pub fn download_size(&self) -> u64 { apt::download_size(&self.ptr.borrow()) }

	/// The amount of space required for installing/removing the packages,"
	///
	/// i.e. the Installed-Size of all packages marked for installation"
	/// minus the Installed-Size of all packages for removal."
	pub fn disk_size(&self) -> DiskSpace {
		let size = apt::disk_size(&self.ptr.borrow());
		if size < 0 {
			return DiskSpace::Free(-size as u64);
		}
		DiskSpace::Require(size as u64)
	}
}

/// Disk Space that `apt` will use for a transaction.
pub enum DiskSpace {
	/// Additional Disk Space required.
	Require(u64),
	/// Disk Space that will be freed
	Free(u64),
}

/// Numeral System for unit conversion.
pub enum NumSys {
	/// Base 2 | 1024 | KibiByte (KiB)
	Binary,
	/// Base 10 | 1000 | KiloByte (KB)
	Decimal,
}

/// Converts bytes into human readable output.
///
/// ```
/// use rust_apt::cache::{unit_str, Cache, NumSys};
/// let cache = Cache::new();
/// let version = cache.get("apt").unwrap().candidate().unwrap();
///
/// println!("{}", unit_str(version.size(), NumSys::Decimal));
/// ```
pub fn unit_str(val: u64, base: NumSys) -> String {
	let val = val as f64;
	let (num, tera, giga, mega, kilo) = match base {
		NumSys::Binary => (1024.0_f64, "TiB", "GiB", "MiB", "KiB"),
		NumSys::Decimal => (1000.0_f64, "TB", "GB", "MB", "KB"),
	};

	let powers = [
		(num.powi(4), tera),
		(num.powi(3), giga),
		(num.powi(2), mega),
		(num, kilo),
	];

	for (divisor, unit) in powers {
		if val > divisor {
			return format!("{:.2} {unit}", val / divisor);
		}
	}
	format!("{val} B")
}

/// Converts seconds into a human readable time string.
pub fn time_str(seconds: u64) -> String {
	if seconds > 60 * 60 * 24 {
		return format!(
			"{}d {}h {}min {}s",
			seconds / 60 / 60 / 24,
			(seconds / 60 / 60) % 24,
			(seconds / 60) % 60,
			seconds % 60,
		);
	}
	if seconds > 60 * 60 {
		return format!(
			"{}h {}min {}s",
			(seconds / 60 / 60) % 24,
			(seconds / 60) % 60,
			seconds % 60,
		);
	}
	if seconds > 60 {
		return format!("{}min {}s", (seconds / 60) % 60, seconds % 60,);
	}
	format!("{seconds}s")
}
