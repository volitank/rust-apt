use std::cell::RefCell;
use std::rc::Rc;

use once_cell::unsync::OnceCell;

use crate::package::Package;
use crate::raw::apt;

#[derive(Debug, Default, PartialEq)]
pub struct PackageSort {
	pub upgradable: bool,
	pub virtual_pkgs: bool,
	pub installed: bool,
	pub auto_installed: bool,
	pub auto_removable: bool,
}

impl PackageSort {
	/// If true, only packages that are upgradable will be included
	pub fn upgradable(mut self, switch: bool) -> Self {
		self.upgradable = switch;
		self
	}

	/// If true, virtual pkgs will be included
	pub fn virtual_pkgs(mut self, switch: bool) -> Self {
		self.virtual_pkgs = switch;
		self
	}

	/// If true, only packages that are installed will be included
	pub fn installed(mut self, switch: bool) -> Self {
		self.installed = switch;
		self
	}

	/// If true, only packages that are auto installed will be included
	pub fn auto_installed(mut self, switch: bool) -> Self {
		self.auto_installed = switch;
		self
	}

	/// If true, only packages that are auto removable will be included
	pub fn auto_removable(mut self, switch: bool) -> Self {
		self.auto_removable = switch;
		self
	}
}

#[derive(Debug, PartialEq)]
pub enum Lookup {
	Desc(*mut apt::DescIterator),
	VerFile(*mut apt::VerFileIterator),
}

#[derive(Debug)]
pub struct Records {
	pub(crate) ptr: *mut apt::PkgRecords,
	pub(crate) pcache: *mut apt::PCache,
	last: RefCell<Option<Lookup>>,
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Records {
	pub fn new(pcache: *mut apt::PCache) -> Self {
		Records {
			ptr: unsafe { apt::pkg_records_create(pcache) },
			pcache,
			last: RefCell::new(None),
		}
	}

	pub fn lookup(&self, record: Lookup) {
		// Check if what we're looking up is currently looked up.
		if let Some(last) = self.last.borrow().as_ref() {
			if last == &record {
				return;
			}
		}

		// Call the correct binding depending on what we're looking up.
		unsafe {
			match &record {
				Lookup::Desc(desc) => {
					apt::desc_file_lookup(self.ptr, *desc);
				},
				Lookup::VerFile(ver_file) => {
					apt::ver_file_lookup(self.ptr, *ver_file);
				},
			}
		}
		// Finally replace the stored value for the next lookup
		self.last.replace(Some(record));
	}

	pub fn description(&self) -> String { unsafe { apt::long_desc(self.ptr) } }

	pub fn summary(&self) -> String { unsafe { apt::short_desc(self.ptr) } }

	pub fn hash_find(&self, hash_type: &str) -> Option<String> {
		unsafe {
			let hash = apt::hash_find(self.ptr, hash_type.to_string());
			if hash == "KeyError" {
				return None;
			}
			Some(hash)
		}
	}
}

impl Drop for Records {
	fn drop(&mut self) {
		unsafe {
			apt::pkg_records_release(self.ptr);
		}
	}
}

/// Internal Struct for managing apt's pkgDepCache.
#[derive(Debug)]
pub struct DepCache {
	ptr: OnceCell<*mut apt::PkgDepCache>,
	pcache: *mut apt::PCache,
}

// DepCache does not have a drop because we don't need to free the pointer.
// The pointer is freed when the cache is dropped
// DepCache is not initialized with the cache as it slows down some operations
// Instead we have this struct to lazily initialize when we need it.
impl DepCache {
	pub fn new(pcache: *mut apt::PCache) -> Self {
		DepCache {
			ptr: OnceCell::new(),
			pcache,
		}
	}

	/// Internal helper to init the depcache if it hasn't been already.
	fn ptr(&self) -> *mut apt::PkgDepCache {
		*self
			.ptr
			.get_or_init(|| unsafe { apt::depcache_create(self.pcache) })
	}

	pub fn clear(&self) {
		unsafe {
			apt::depcache_create(self.pcache);
		}
	}

	pub fn is_upgradable(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		unsafe { apt::pkg_is_upgradable(self.ptr(), pkg_ptr) }
	}

	pub fn is_auto_installed(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		unsafe { apt::pkg_is_auto_installed(self.ptr(), pkg_ptr) }
	}

	pub fn is_auto_removable(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		let dep_ptr = self.ptr();
		unsafe {
			(apt::pkg_is_installed(pkg_ptr) || apt::pkg_marked_install(dep_ptr, pkg_ptr))
				&& apt::pkg_is_garbage(self.ptr(), pkg_ptr)
		}
	}

	pub fn marked_install(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		unsafe { apt::pkg_marked_install(self.ptr(), pkg_ptr) }
	}

	pub fn marked_upgrade(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		unsafe { apt::pkg_marked_upgrade(self.ptr(), pkg_ptr) }
	}

	pub fn marked_delete(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		unsafe { apt::pkg_marked_delete(self.ptr(), pkg_ptr) }
	}

	pub fn marked_keep(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		unsafe { apt::pkg_marked_keep(self.ptr(), pkg_ptr) }
	}

	pub fn marked_downgrade(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		unsafe { apt::pkg_marked_downgrade(self.ptr(), pkg_ptr) }
	}

	pub fn marked_reinstall(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		unsafe { apt::pkg_marked_reinstall(self.ptr(), pkg_ptr) }
	}

	pub fn is_now_broken(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		unsafe { apt::pkg_is_now_broken(self.ptr(), pkg_ptr) }
	}

	pub fn is_inst_broken(&self, pkg_ptr: &apt::PackagePtr) -> bool {
		unsafe { apt::pkg_is_inst_broken(self.ptr(), pkg_ptr) }
	}
}

#[derive(Debug)]
pub struct Cache {
	pub ptr: *mut apt::PCache,
	pub records: Rc<RefCell<Records>>,
	depcache: Rc<RefCell<DepCache>>,
}

impl Drop for Cache {
	fn drop(&mut self) {
		unsafe {
			apt::pkg_cache_release(self.ptr);
		}
	}
}

impl Default for Cache {
	fn default() -> Self { Self::new() }
}

impl Cache {
	/// Initialize the configuration system, open and return the cache.
	///
	/// This is the entry point for all operations of this crate.
	pub fn new() -> Self {
		apt::init_config_system();
		let cache_ptr = apt::pkg_cache_create();
		Self {
			ptr: cache_ptr,
			records: Rc::new(RefCell::new(Records::new(cache_ptr))),
			depcache: Rc::new(RefCell::new(DepCache::new(cache_ptr))),
		}
	}

	/// Clears all changes made to packages.
	///
	/// Currently this doesn't do anything as we can't manipulate packages.
	pub fn clear(&self) { self.depcache.borrow().clear(); }

	/// Returns an iterator of SourceURIs.
	///
	/// These are the files that `apt update` will fetch.
	pub fn sources(&self) -> impl Iterator<Item = apt::SourceFile> + '_ {
		unsafe { apt::source_uris(self.ptr).into_iter() }
	}

	/// Returns an iterator of Packages that provide the virtual package
	pub fn provides(
		&self,
		virt_pkg: &Package,
		cand_only: bool,
	) -> impl Iterator<Item = Package> + '_ {
		unsafe {
			apt::pkg_provides_list(self.ptr, &virt_pkg.ptr, cand_only)
				.into_iter()
				.map(|pkg| Package::new(Rc::clone(&self.records), Rc::clone(&self.depcache), pkg))
		}
	}

	// Disabled as it doesn't really work yet. Would likely need to
	// Be on the objects them self and not the cache
	// pub fn validate(&self, ver: *mut apt::VerIterator) -> bool {
	// 	unsafe { apt::validate(ver, self._cache) }
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
		unsafe {
			if !arch.is_empty() {
				return apt::pkg_cache_find_name_arch(self.ptr, name.to_owned(), arch.to_owned());
			}
			apt::pkg_cache_find_name(self.ptr, name.to_owned())
		}
	}

	/// An iterator of packages sorted by package name.
	///
	/// Slower than the `cache.packages` method.
	pub fn sorted<'a>(&'a self, sort: &'a PackageSort) -> impl Iterator<Item = Package> + '_ {
		let mut pkgs = self.packages(sort).collect::<Vec<Package>>();
		pkgs.sort_by_cached_key(|pkg| pkg.name.to_owned());
		pkgs.into_iter()
	}

	/// An iterator of packages not sorted by name.
	///
	/// Faster than the `cache.sorted` method.
	pub fn packages<'a>(&'a self, sort: &'a PackageSort) -> impl Iterator<Item = Package> + '_ {
		unsafe {
			apt::pkg_list(self.ptr)
				.into_iter()
				.filter_map(move |pkg_ptr| self.sort_package(pkg_ptr, sort))
		}
	}

	/// Internal method for sorting packages.
	fn sort_package(&self, pkg_ptr: apt::PackagePtr, sort: &PackageSort) -> Option<Package> {
		unsafe {
			if (!sort.virtual_pkgs && !apt::pkg_has_versions(&pkg_ptr))
				|| (sort.upgradable && !self.depcache.borrow().is_upgradable(&pkg_ptr))
				|| (sort.installed && !apt::pkg_is_installed(&pkg_ptr))
				|| (sort.auto_installed && !self.depcache.borrow().is_auto_installed(&pkg_ptr))
				|| (sort.auto_removable && !self.depcache.borrow().is_auto_removable(&pkg_ptr))
			{
				return None;
			}
		}
		Some(Package::new(
			Rc::clone(&self.records),
			Rc::clone(&self.depcache),
			pkg_ptr,
		))
	}
}

/// Converts a version's size into human readable output.
///
/// `println!("{}", unit_str(version.size))`
pub fn unit_str(val: i32) -> String {
	let num: i32 = 1000;
	if val > num.pow(2) {
		return format!("{:.2} MB", val as f32 / 1000.0 / 1000.0);
	} else if val > num {
		return format!("{:.2} kB", val as f32 / 1000.0);
	}
	return format!("{val} B");
}
