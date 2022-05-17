use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::{ffi, fmt};

use once_cell::unsync::OnceCell;

#[deny(clippy::not_unsafe_ptr_arg_deref)]
use crate::raw;
use crate::raw::apt;

#[derive(Debug)]
pub struct Package<'a> {
	// Commented fields are to be implemented
	_lifetime: &'a PhantomData<Cache>,
	records: Rc<RefCell<Records>>,
	ptr: *mut apt::PkgIterator,
	pub name: String,
	pub arch: String,
	pub id: i32,
	pub essential: bool,
	pub current_state: i32,
	pub inst_state: i32,
	pub selected_state: i32,
	pub has_versions: bool,
	pub has_provides: bool,
	// provides_list: List[Tuple[str, str, Version]],
}

impl<'a> Package<'a> {
	pub fn new(records: Rc<RefCell<Records>>, pkg_ptr: *mut apt::PkgIterator) -> Package<'a> {
		unsafe {
			Package {
				_lifetime: &PhantomData,
				ptr: pkg_ptr,
				records,
				name: apt::get_fullname(pkg_ptr, true),
				arch: raw::own_string(apt::pkg_arch(pkg_ptr)).unwrap(),
				id: apt::pkg_id(pkg_ptr),
				essential: apt::pkg_essential(pkg_ptr),
				current_state: apt::pkg_current_state(pkg_ptr),
				inst_state: apt::pkg_inst_state(pkg_ptr),
				selected_state: apt::pkg_selected_state(pkg_ptr),
				has_versions: apt::pkg_has_versions(pkg_ptr),
				has_provides: apt::pkg_has_provides(pkg_ptr),
			}
		}
	}

	/// Get the fullname of the package.
	///
	/// Pretty is a bool that will omit the native arch.
	///
	/// For example on an amd64 system:
	///
	/// `pkg.get_fullname(true)` would return just `"apt"` for the amd64 package
	/// and `"apt:i386"` for the i386 package.
	///
	/// `pkg.get_fullname(false)` would return `"apt:amd64"` for the amd64
	/// version and `"apt:i386"` for the i386 package.
	pub fn get_fullname(&self, pretty: bool) -> String {
		unsafe { apt::get_fullname(self.ptr, pretty) }
	}

	/// Returns the version object of the candidate.
	///
	/// If there isn't a candidate, returns None
	pub fn candidate(&self) -> Option<Version<'a>> {
		unsafe {
			let ver = apt::pkg_candidate_version(self.records.borrow_mut().pcache, self.ptr);
			if apt::ver_end(ver) {
				return None;
			}
			Some(Version::new(Rc::clone(&self.records), ver, false))
		}
	}

	/// Check if a package is installed.
	pub fn installed(&self) -> Option<Version<'a>> {
		unsafe {
			let ver = apt::pkg_current_version(self.ptr);
			if apt::ver_end(ver) {
				return None;
			}
			Some(Version::new(Rc::clone(&self.records), ver, false))
		}
	}

	/// Check if a package is upgradable.
	pub fn is_upgradable(&self) -> bool {
		unsafe { apt::pkg_is_upgradable(self.records.borrow_mut().pcache, self.ptr) }
	}

	/// Returns a version list starting with the newest and ending with the
	/// oldest.
	pub fn versions(&self) -> Vec<Version<'a>> {
		let mut version_map = Vec::new();
		unsafe {
			let version_iterator = apt::pkg_version_list(self.ptr);
			let mut first = true;
			loop {
				if !first {
					apt::ver_next(version_iterator);
				}
				first = false;
				if apt::ver_end(version_iterator) {
					break;
				}
				version_map.push(Version::new(
					Rc::clone(&self.records),
					version_iterator,
					true,
				));
			}
			apt::ver_release(version_iterator);
		}
		version_map
	}
}

// We must release the pointer on drop
impl<'a> Drop for Package<'a> {
	fn drop(&mut self) {
		unsafe {
			apt::pkg_release(self.ptr);
		}
	}
}

impl<'a> fmt::Display for Package<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"Package< name: {}, arch: {}, id: {}, essential: {}, states: [curr: {}, inst {}, sel \
			 {}], virtual: {}, provides: {}>",
			self.name,
			self.arch,
			self.id,
			self.essential,
			self.current_state,
			self.inst_state,
			self.selected_state,
			!self.has_versions,
			self.has_provides
		)?;
		Ok(())
	}
}

#[derive(Debug)]
struct PackageFile {
	ver_file: *mut apt::VerFileIterator,
	pkg_file: *mut apt::PkgFileIterator,
	index: *mut apt::PkgIndexFile,
}

impl Drop for PackageFile {
	fn drop(&mut self) {
		unsafe {
			apt::ver_file_release(self.ver_file);
			apt::pkg_file_release(self.pkg_file);
			apt::pkg_index_file_release(self.index);
		}
	}
}

impl fmt::Display for PackageFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "package file: {:?}", self.pkg_file)?;
		Ok(())
	}
}

#[derive(Debug)]
pub struct Version<'a> {
	//_parent: RefCell<Package<'a>>,
	_lifetime: &'a PhantomData<Cache>,
	_records: Rc<RefCell<Records>>,
	desc_ptr: *mut apt::DescIterator,
	ptr: *mut apt::VerIterator,
	file_list: OnceCell<Vec<PackageFile>>,
	pub pkgname: String,
	pub version: String,
	pub size: i32,
	pub installed_size: i32,
	pub arch: String,
	pub downloadable: bool,
	pub id: i32,
	pub section: String,
	pub priority: i32,
	pub priority_str: String,
	// provides_list: List[Tuple[str,str,str]]
	// depends_list: Dict[str, List[List[Dependency]]]
	// parent_pkg: Package
	// multi_arch: int
	// MULTI_ARCH_ALL: int
	// MULTI_ARCH_ALLOWED: int
	// MULTI_ARCH_ALL_ALLOWED: int
	// MULTI_ARCH_ALL_FOREIGN: int
	// MULTI_ARCH_FOREIGN: int
	// MULTI_ARCH_NO: int
	// MULTI_ARCH_NONE: int
	// MULTI_ARCH_SAME: int
}

impl<'a> Version<'a> {
	fn new(records: Rc<RefCell<Records>>, ver_ptr: *mut apt::VerIterator, clone: bool) -> Self {
		unsafe {
			let ver_priority = apt::ver_priority(records.borrow_mut().pcache, ver_ptr);
			Self {
				ptr: if clone { apt::ver_clone(ver_ptr) } else { ver_ptr },
				desc_ptr: apt::ver_desc_file(ver_ptr),
				_records: records,
				_lifetime: &PhantomData,
				pkgname: apt::ver_name(ver_ptr),
				priority: ver_priority,
				file_list: OnceCell::new(),
				version: raw::own_string(apt::ver_str(ver_ptr)).unwrap(),
				size: apt::ver_size(ver_ptr),
				installed_size: apt::ver_installed_size(ver_ptr),
				arch: raw::own_string(apt::ver_arch(ver_ptr)).unwrap(),
				downloadable: apt::ver_downloadable(ver_ptr),
				id: apt::ver_id(ver_ptr),
				section: raw::own_string(apt::ver_section(ver_ptr))
					.unwrap_or_else(|| String::from("None")),
				priority_str: raw::own_string(apt::ver_priority_str(ver_ptr)).unwrap(),
			}
		}
	}

	/// Internal Method for Generating the PackageFiles
	fn gen_file_list(&self) -> Vec<PackageFile> {
		let mut package_files = Vec::new();
		unsafe {
			let ver_file = apt::ver_file(self.ptr);
			let mut first = true;

			loop {
				if !first {
					apt::ver_file_next(ver_file);
				}

				first = false;
				if apt::ver_file_end(ver_file) {
					break;
				}
				let pkg_file = apt::ver_pkg_file(ver_file);
				package_files.push(PackageFile {
					ver_file: apt::ver_file_clone(ver_file),
					pkg_file,
					index: apt::pkg_index_file(self._records.borrow_mut().pcache, pkg_file),
				});
			}
			apt::ver_file_release(ver_file);
		}
		package_files
	}

	/// Check if the version is installed
	pub fn is_installed(&self) -> bool { unsafe { apt::ver_installed(self.ptr) } }

	/// Get the translated long description
	pub fn description(&self) -> String {
		let records = self._records.borrow_mut();
		records.lookup(Lookup::Desc(self.desc_ptr));
		records.description()
	}

	/// Get the translated short description
	pub fn summary(&self) -> String {
		let records = self._records.borrow_mut();
		records.lookup(Lookup::Desc(self.desc_ptr));
		records.summary()
	}

	/// Get the sha256 hash. If there isn't one returns None
	/// This is equivalent to `version.hash("sha256")`
	pub fn sha256(&self) -> Option<String> { self.hash("sha256") }

	/// Get the sha512 hash. If there isn't one returns None
	/// This is equivalent to `version.hash("sha512")`
	pub fn sha512(&self) -> Option<String> { self.hash("sha512") }

	/// Get the hash specified. If there isn't one returns None
	/// `version.hash("md5sum")`
	pub fn hash(&self, hash_type: &str) -> Option<String> {
		let package_files = self.file_list.get_or_init(|| self.gen_file_list());

		if let Some(pkg_file) = package_files.into_iter().next() {
			let records = self._records.borrow_mut();
			records.lookup(Lookup::VerFile(pkg_file.ver_file));
			return records.hash_find(hash_type);
		}
		None
	}

	/// Returns an iterator of URIs for the version
	pub fn uris(&'a self) -> impl Iterator<Item = String> + 'a {
		self.file_list
			.get_or_init(|| self.gen_file_list())
			.into_iter()
			.filter_map(|package_file| {
				let records = self._records.borrow_mut();
				records.lookup(Lookup::VerFile(package_file.ver_file));

				let uri = unsafe { apt::ver_uri(records.ptr, package_file.index) };
				if !uri.starts_with("file:") {
					Some(uri)
				} else {
					None
				}
			})
	}
}

// We must release the pointer on drop
impl<'a> Drop for Version<'a> {
	fn drop(&mut self) {
		unsafe {
			apt::ver_release(self.ptr);
			apt::ver_desc_release(self.desc_ptr)
		}
	}
}

impl<'a> fmt::Display for Version<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{}: Version {} <ID: {}, arch: {}, size: {}, installed_size: {}, section: {} Priority \
			 {} at {}, downloadable: {}>",
			self.pkgname,
			self.version,
			self.id,
			self.arch,
			unit_str(self.size),
			unit_str(self.installed_size),
			self.section,
			self.priority_str,
			self.priority,
			self.downloadable,
		)?;
		Ok(())
	}
}

#[derive(Debug, Default, PartialEq)]
pub struct PackageSort {
	pub upgradable: bool,
	pub virtual_pkgs: bool,
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
}

#[derive(Debug, PartialEq)]
pub enum Lookup {
	Desc(*mut apt::DescIterator),
	VerFile(*mut apt::VerFileIterator),
}

#[derive(Debug)]
pub struct Records {
	ptr: *mut apt::PkgRecords,
	pcache: *mut apt::PCache,
	last: RefCell<Option<Lookup>>,
}

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
			} else {
				return Some(hash);
			}
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

#[derive(Debug)]
pub struct Cache {
	pub ptr: *mut apt::PCache,
	pub records: Rc<RefCell<Records>>,
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
		}
	}

	/// Clears all changes made to packages.
	///
	/// Currently this doesn't do anything as we can't manipulate packages.
	pub fn clear(&mut self) {
		unsafe {
			apt::depcache_init(self.ptr);
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

		unsafe {
			if apt::pkg_end(pkg_ptr) {
				apt::pkg_release(pkg_ptr);
				return None;
			}
		}
		Some(Package::new(Rc::clone(&self.records), pkg_ptr))
	}

	/// Internal method for getting a package by name
	///
	/// Find a package by name and additionally architecture.
	///
	/// The returned iterator will either be at the end, or at a matching
	/// package.
	fn find_by_name(&self, name: &str, arch: &str) -> *mut apt::PkgIterator {
		unsafe {
			let name = ffi::CString::new(name).unwrap();
			if !arch.is_empty() {
				let arch = ffi::CString::new(arch).unwrap();
				return apt::pkg_cache_find_name_arch(self.ptr, name.as_ptr(), arch.as_ptr());
			}
			apt::pkg_cache_find_name(self.ptr, name.as_ptr())
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
		Self::pointers(unsafe { apt::pkg_begin(self.ptr) })
			.filter_map(move |pkg_ptr| self.sort_package(pkg_ptr, sort))
	}

	/// Internal method for sorting packages.
	fn sort_package(&self, pkg_ptr: *mut apt::PkgIterator, sort: &PackageSort) -> Option<Package> {
		unsafe {
			if (!sort.virtual_pkgs && !apt::pkg_has_versions(pkg_ptr))
				|| (sort.upgradable && !apt::pkg_is_upgradable(self.ptr, pkg_ptr))
			{
				apt::pkg_release(pkg_ptr);
				return None;
			}
		}
		Some(Package::new(Rc::clone(&self.records), pkg_ptr))
	}

	/// Internal method for iterating apt's package pointers.
	fn pointers(iter_ptr: *mut apt::PkgIterator) -> impl Iterator<Item = *mut apt::PkgIterator> {
		unsafe {
			std::iter::from_fn(move || {
				if apt::pkg_end(iter_ptr) {
					apt::pkg_release(iter_ptr);
					return None;
				}

				let current = apt::pkg_clone(iter_ptr);
				apt::pkg_next(iter_ptr);
				Some(current)
			})
		}
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
