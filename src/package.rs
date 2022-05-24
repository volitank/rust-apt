use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use once_cell::unsync::OnceCell;

use crate::cache::{unit_str, Cache, DepCache, Lookup, Records};
use crate::raw::apt;

#[derive(Debug)]
pub struct Package<'a> {
	// Commented fields are to be implemented
	_lifetime: &'a PhantomData<Cache>,
	records: Rc<RefCell<Records>>,
	depcache: Rc<RefCell<DepCache>>,
	pub(crate) ptr: *mut apt::PkgIterator,
	pub name: String,
	pub arch: String,
	pub id: i32,
	pub essential: bool,
	pub current_state: i32,
	pub inst_state: i32,
	pub selected_state: i32,
	pub has_versions: bool,
	pub has_provides: bool,
}

impl<'a> Package<'a> {
	pub fn new(
		records: Rc<RefCell<Records>>,
		depcache: Rc<RefCell<DepCache>>,
		pkg_ptr: *mut apt::PkgIterator,
	) -> Package<'a> {
		unsafe {
			Package {
				_lifetime: &PhantomData,
				ptr: pkg_ptr,
				records,
				depcache,
				name: apt::get_fullname(pkg_ptr, true),
				arch: apt::pkg_arch(pkg_ptr),
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

	/// Returns the version object of the installed version.
	///
	/// If there isn't an installed version, returns None
	pub fn installed(&self) -> Option<Version<'a>> {
		unsafe {
			let ver = apt::pkg_current_version(self.ptr);
			if apt::ver_end(ver) {
				return None;
			}
			Some(Version::new(Rc::clone(&self.records), ver, false))
		}
	}

	/// Check if the package is installed.
	pub fn is_installed(&self) -> bool { unsafe { apt::pkg_is_installed(self.ptr) } }

	/// Check if the package is upgradable.
	pub fn is_upgradable(&self) -> bool { self.depcache.borrow().is_upgradable(self.ptr) }

	/// Check if the package is auto installed. (Not installed by the user)
	pub fn is_auto_installed(&self) -> bool { self.depcache.borrow().is_auto_installed(self.ptr) }

	/// Check if the package is auto removable
	pub fn is_auto_removable(&self) -> bool { self.depcache.borrow().is_auto_removable(self.ptr) }

	/// Check if the package is now broken
	pub fn is_now_broken(&self) -> bool { self.depcache.borrow().is_now_broken(self.ptr) }

	/// Check if the package package installed is broken
	pub fn is_inst_broken(&self) -> bool { self.depcache.borrow().is_inst_broken(self.ptr) }

	/// Check if the package is marked install
	pub fn marked_install(&self) -> bool { self.depcache.borrow().marked_install(self.ptr) }

	/// Check if the package is marked upgrade
	pub fn marked_upgrade(&self) -> bool { self.depcache.borrow().marked_upgrade(self.ptr) }

	/// Check if the package is marked delete
	pub fn marked_delete(&self) -> bool { self.depcache.borrow().marked_delete(self.ptr) }

	/// Check if the package is marked keep
	pub fn marked_keep(&self) -> bool { self.depcache.borrow().marked_keep(self.ptr) }

	/// Check if the package is marked downgrade
	pub fn marked_downgrade(&self) -> bool { self.depcache.borrow().marked_downgrade(self.ptr) }

	/// Check if the package is marked reinstall
	pub fn marked_reinstall(&self) -> bool { self.depcache.borrow().marked_reinstall(self.ptr) }

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
				version: apt::ver_str(ver_ptr),
				size: apt::ver_size(ver_ptr),
				installed_size: apt::ver_installed_size(ver_ptr),
				arch: apt::ver_arch(ver_ptr),
				downloadable: apt::ver_downloadable(ver_ptr),
				id: apt::ver_id(ver_ptr),
				section: apt::ver_section(ver_ptr),
				priority_str: apt::ver_priority_str(ver_ptr),
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
