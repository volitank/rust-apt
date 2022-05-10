use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::{ffi, fmt};

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
	pub fn new(
		records: Rc<RefCell<Records>>,
		pkg_ptr: *mut apt::PkgIterator,
		clone: bool,
	) -> Package<'a> {
		unsafe {
			Package {
				// Get a new pointer for the package
				ptr: if clone { apt::pkg_clone(pkg_ptr) } else { pkg_ptr },
				_lifetime: &PhantomData,
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

	pub fn get_fullname(&self, pretty: bool) -> String {
		unsafe { apt::get_fullname(self.ptr, pretty) }
	}

	pub fn candidate(&self) -> Option<Version<'a>> {
		unsafe {
			let ver = apt::pkg_candidate_version(self.records.borrow_mut().pcache, self.ptr);
			if apt::ver_end(ver) {
				return None;
			}
			Some(Version::new(Rc::clone(&self.records), ver, false))
		}
	}

	pub fn installed(&self) -> Option<Version<'a>> {
		unsafe {
			let ver = apt::pkg_current_version(self.ptr);
			if apt::ver_end(ver) {
				return None;
			}
			Some(Version::new(Rc::clone(&self.records), ver, false))
		}
	}

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
	pub pkgname: String,
	pub version: String,
	// hash: int
	// 	_file_list: Option<Vec<PackageFile>>,
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
				//_file_list: None,
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

	fn file_list(&self) -> Vec<PackageFile> {
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

	pub fn is_installed(&self) -> bool { unsafe { apt::ver_installed(self.ptr) } }

	pub fn description(&self) -> String {
		let records = self._records.borrow_mut();
		records.lookup(Lookup::Desc(self.desc_ptr));
		records.description()
	}

	pub fn summary(&self) -> String {
		let records = self._records.borrow_mut();
		records.lookup(Lookup::Desc(self.desc_ptr));
		records.summary()
	}

	pub fn get_uris(&self) -> Vec<String> {
		let mut uris = Vec::new();
		for package_file in self.file_list() {
			unsafe {
				let records = self._records.borrow_mut();
				records.lookup(Lookup::VerFile(package_file.ver_file));

				let uri = apt::ver_uri(records.ptr, package_file.index);
				if !uri.starts_with("file:") {
					uris.push(apt::ver_uri(records.ptr, package_file.index));
				}
			}
		}
		uris
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

#[derive(Default)]
pub struct PackageSort {
	pub upgradable: bool,
	pub virtual_pkgs: bool,
}

// impl Default for PackageSort {
// 	fn default() -> PackageSort{
// 		PackageSort {
// 			iInsertMax: -1,
// 			iUpdateMax: -1,
// 			iDeleteMax: -1,
// 			iInstanceMax: -1,
// 			tFirstInstance: false,
// 			tCreateTables: false,
// 			tContinue: false,
// 		}
// 	}
// }
#[derive(Debug, PartialEq)]
pub enum Lookup {
	Desc(*mut apt::DescIterator),
	VerFile(*mut apt::VerFileIterator),
}

#[derive(Debug)]
pub struct Records {
	ptr: *mut apt::PkgRecords,
	pcache: *mut apt::PCache,
	last: RefCell<Option<Lookup>>, // pub _helper: RefCell<String>,
}

impl Records {
	pub fn new(pcache: *mut apt::PCache) -> Self {
		Records {
			ptr: unsafe { apt::pkg_records_create(pcache) },
			pcache: pcache,
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
	pointers: Vec<*mut apt::PkgIterator>,
	pub records: Rc<RefCell<Records>>,
}

impl Drop for Cache {
	fn drop(&mut self) {
		unsafe {
			apt::pkg_cache_release(self.ptr);
			for ptr in (*self.pointers).iter() {
				apt::pkg_release(*ptr);
			}
		}
	}
}

impl Default for Cache {
	fn default() -> Self { Self::new() }
}

impl Cache {
	pub fn new() -> Self {
		unsafe {
			apt::init_config_system();
			let cache_ptr = apt::pkg_cache_create();
			Self {
				ptr: cache_ptr,
				pointers: Cache::get_pointers(apt::pkg_begin(cache_ptr)),
				records: Rc::new(RefCell::new(Records::new(cache_ptr))),
			}
		}
	}

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

	pub fn get<'a>(&'a self, name: &str) -> Option<Package<'a>> {
		let _name: &str;
		let _arch: &str;

		if name.contains(':') {
			let package: Vec<&str> = name.split(':').collect();

			if package.len() > 2 {
				panic!("Value is wrong");
			}

			_name = package[0];
			_arch = package[1];
		} else {
			_name = name;
			_arch = "";
		}

		let pkg_ptr = self.find_by_name(_name, _arch);
		unsafe {
			if apt::pkg_end(pkg_ptr) {
				apt::pkg_release(pkg_ptr);
				return None;
			}
		}
		Some(Package::new(Rc::clone(&self.records), pkg_ptr, false))
	}

	/// Internal method for getting a package by name
	/// Find a package by name and additionally architecture.
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

	pub fn sorted(&self, sort: PackageSort) -> BTreeMap<String, Package> {
		let mut package_map = BTreeMap::new();
		unsafe {
			let pkg_iterator = apt::pkg_begin(self.ptr);
			let mut first = true;
			loop {
				// We have to make sure we get the first package
				if !first {
					apt::pkg_next(pkg_iterator);
				}
				first = false;
				if apt::pkg_end(pkg_iterator) {
					break;
				}

				if !sort.virtual_pkgs && !apt::pkg_has_versions(pkg_iterator) {
					continue;
				}
				if sort.upgradable && !apt::pkg_is_upgradable(self.ptr, pkg_iterator) {
					continue;
				}
				package_map.insert(
					apt::get_fullname(pkg_iterator, true),
					Package::new(Rc::clone(&self.records), pkg_iterator, true),
				);
			}
			apt::pkg_release(pkg_iterator);
		}
		package_map
	}

	pub fn packages(&self) -> impl Iterator<Item = Package> + '_ {
		let pointers = &self.pointers;
		pointers
			.iter()
			.map(|pkg_ptr| Package::new(Rc::clone(&self.records), *pkg_ptr, true))
	}

	fn get_pointers(pkg_iterator: *mut apt::PkgIterator) -> Vec<*mut apt::PkgIterator> {
		let mut package_map = Vec::new();
		unsafe {
			let mut first = true;
			loop {
				// We have to make sure we get the first package
				if !first {
					apt::pkg_next(pkg_iterator);
				}

				first = false;
				if apt::pkg_end(pkg_iterator) {
					break;
				}
				package_map.push(apt::pkg_clone(pkg_iterator));
			}
			apt::pkg_release(pkg_iterator);
		}
		package_map
	}
}

pub fn unit_str(val: i32) -> String {
	let num: i32 = 1000;
	if val > num.pow(2) {
		return format!("{:.2} MB", val as f32 / 1000.0 / 1000.0);
	} else if val > num {
		return format!("{:.2} kB", val as f32 / 1000.0);
	}
	return format!("{val} B");
}
