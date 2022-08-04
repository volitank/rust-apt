//! Contains Package, Version and Dependency Structs.
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use once_cell::unsync::OnceCell;

use crate::cache::raw::{pkg_version_list, ver_file_list, ver_pkg_file_list};
use crate::cache::{Cache, PackageFile};
use crate::depcache::DepCache;
use crate::records::Records;
use crate::util::{cmp_versions, unit_str, NumSys};

/// A struct representing an `apt` Package
#[derive(Debug)]
pub struct Package<'a> {
	_lifetime: &'a PhantomData<Cache>,
	records: Rc<RefCell<Records>>,
	depcache: Rc<RefCell<DepCache>>,
	pub(crate) ptr: raw::PackagePtr,
}

impl<'a> Package<'a> {
	pub fn new(
		records: Rc<RefCell<Records>>,
		depcache: Rc<RefCell<DepCache>>,
		pkg_ptr: raw::PackagePtr,
	) -> Package<'a> {
		Package {
			_lifetime: &PhantomData,
			records,
			depcache,
			ptr: pkg_ptr,
		}
	}

	/// Get the fullname of the package.
	///
	/// Pretty is a bool that will omit the native arch.
	///
	/// For example on an amd64 system:
	///
	/// ```
	/// use rust_apt::cache::Cache;
	/// let cache = Cache::new();
	/// if let Some(pkg) = cache.get("apt") {
	///    // Prints just "apt"
	///    println!("{}", pkg.fullname(true));
	///    // Prints "apt:amd64"
	///    println!("{}", pkg.fullname(false));
	/// };
	///
	/// if let Some(pkg) = cache.get("apt:i386") {
	///    // Prints "apt:i386" for the i386 package
	///    println!("{}", pkg.fullname(true));
	/// };
	/// ```
	pub fn fullname(&self, pretty: bool) -> String { raw::get_fullname(&self.ptr, pretty) }

	/// Return the name of the package without the architecture
	pub fn name(&self) -> String { raw::pkg_name(&self.ptr) }

	/// Get the architecture of the package.
	pub fn arch(&self) -> String { raw::pkg_arch(&self.ptr) }

	/// Get the ID of the package.
	pub fn id(&self) -> u32 { raw::pkg_id(&self.ptr) }

	/// The current state of the package.
	pub fn current_state(&self) -> u8 { raw::pkg_current_state(&self.ptr) }

	/// The installed state of the package.
	pub fn inst_state(&self) -> u8 { raw::pkg_inst_state(&self.ptr) }

	/// The selected state of the package.
	pub fn selected_state(&self) -> u8 { raw::pkg_selected_state(&self.ptr) }

	/// Check if the package is essnetial or not.
	pub fn essential(&self) -> bool { raw::pkg_essential(&self.ptr) }

	/// Check if the package has versions.
	pub fn has_versions(&self) -> bool { raw::pkg_has_versions(&self.ptr) }

	/// Check if the package has provides.
	pub fn has_provides(&self) -> bool { raw::pkg_has_provides(&self.ptr) }

	/// Returns the version object of the candidate.
	///
	/// If there isn't a candidate, returns None
	pub fn candidate(&self) -> Option<Version<'a>> {
		let ver = raw::pkg_candidate_version(&self.records.borrow().cache.borrow(), &self.ptr);
		if ver.ptr.is_null() {
			return None;
		}
		Some(Version::new(
			Rc::clone(&self.records),
			Rc::clone(&self.depcache),
			ver,
		))
	}

	/// Returns the version object of the installed version.
	///
	/// If there isn't an installed version, returns None
	pub fn installed(&self) -> Option<Version<'a>> {
		let ver = raw::pkg_current_version(&self.ptr);
		if ver.ptr.is_null() {
			return None;
		}
		Some(Version::new(
			Rc::clone(&self.records),
			Rc::clone(&self.depcache),
			ver,
		))
	}

	/// Check if the package is installed.
	pub fn is_installed(&self) -> bool { raw::pkg_is_installed(&self.ptr) }

	/// Check if the package is upgradable.
	///
	/// `skip_depcache = true` increases performance by skipping the pkgDepCache
	/// Skipping the depcache is very unnecessary if it's already been
	/// initialized If you're not sure, set `skip_depcache = false`
	pub fn is_upgradable(&self, skip_depcache: bool) -> bool {
		self.depcache
			.borrow()
			.is_upgradable(&self.ptr, skip_depcache)
	}

	/// Check if the package is auto installed. (Not installed by the user)
	pub fn is_auto_installed(&self) -> bool { self.depcache.borrow().is_auto_installed(&self.ptr) }

	/// Check if the package is auto removable
	pub fn is_auto_removable(&self) -> bool { self.depcache.borrow().is_auto_removable(&self.ptr) }

	/// Check if the package is now broken
	pub fn is_now_broken(&self) -> bool { self.depcache.borrow().is_now_broken(&self.ptr) }

	/// Check if the package package installed is broken
	pub fn is_inst_broken(&self) -> bool { self.depcache.borrow().is_inst_broken(&self.ptr) }

	/// Check if the package is marked install
	pub fn marked_install(&self) -> bool { self.depcache.borrow().marked_install(&self.ptr) }

	/// Check if the package is marked upgrade
	pub fn marked_upgrade(&self) -> bool { self.depcache.borrow().marked_upgrade(&self.ptr) }

	/// Check if the package is marked delete
	pub fn marked_delete(&self) -> bool { self.depcache.borrow().marked_delete(&self.ptr) }

	/// Check if the package is marked keep
	pub fn marked_keep(&self) -> bool { self.depcache.borrow().marked_keep(&self.ptr) }

	/// Check if the package is marked downgrade
	pub fn marked_downgrade(&self) -> bool { self.depcache.borrow().marked_downgrade(&self.ptr) }

	/// Check if the package is marked reinstall
	pub fn marked_reinstall(&self) -> bool { self.depcache.borrow().marked_reinstall(&self.ptr) }

	/// Returns a version list starting with the newest and ending with the
	/// oldest.
	pub fn versions(&self) -> impl Iterator<Item = Version<'a>> + '_ {
		pkg_version_list(&self.ptr).into_iter().map(|ver_ptr| {
			Version::new(Rc::clone(&self.records), Rc::clone(&self.depcache), ver_ptr)
		})
	}
}

impl<'a> fmt::Display for Package<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"Package< name: {}, arch: {}, id: {}, essential: {}, states: [curr: {}, inst {}, sel \
			 {}], virtual: {}, provides: {}>",
			self.name(),
			self.arch(),
			self.id(),
			self.essential(),
			self.current_state(),
			self.inst_state(),
			self.selected_state(),
			!self.has_versions(),
			self.has_provides(),
		)?;
		Ok(())
	}
}

// Implementations for comparing packages.
impl<'a> PartialEq for Package<'a> {
	fn eq(&self, other: &Self) -> bool { self.id() == other.id() }
}

/// A struct representing a version of an `apt` Package
#[derive(Debug)]
pub struct Version<'a> {
	_lifetime: &'a PhantomData<Cache>,
	ptr: raw::VersionPtr,
	records: Rc<RefCell<Records>>,
	depcache: Rc<RefCell<DepCache>>,
	depends_list: OnceCell<HashMap<String, Vec<Dependency>>>,
}

impl<'a> Version<'a> {
	fn new(
		records: Rc<RefCell<Records>>,
		depcache: Rc<RefCell<DepCache>>,
		ver_ptr: raw::VersionPtr,
	) -> Self {
		Self {
			_lifetime: &PhantomData,
			records,
			depcache,
			depends_list: OnceCell::new(),
			ptr: ver_ptr,
		}
	}

	/// Return the version's parent package.
	pub fn parent(&self) -> Package {
		Package::new(
			Rc::clone(&self.records),
			Rc::clone(&self.depcache),
			raw::ver_parent(&self.ptr),
		)
	}

	/// The architecture of the version.
	pub fn arch(&self) -> String { raw::ver_arch(&self.ptr) }

	/// The version string of the version. "1.4.10"
	pub fn version(&self) -> String { raw::ver_str(&self.ptr) }

	/// The section of the version as shown in `apt show`.
	pub fn section(&self) -> String { raw::ver_section(&self.ptr) }

	/// The priority string as shown in `apt show`.
	pub fn priority_str(&self) -> String { raw::ver_priority_str(&self.ptr) }

	/// The name of the source package the version was built from.
	pub fn source_name(&self) -> String { raw::ver_source_name(&self.ptr) }

	/// The version of the source package.
	pub fn source_version(&self) -> String { raw::ver_source_version(&self.ptr) }

	/// The priority of the package as shown in `apt policy`.
	pub fn priority(&self) -> i32 {
		raw::ver_priority(&self.records.borrow().cache.borrow(), &self.ptr)
	}

	/// The size of the .deb file.
	pub fn size(&self) -> u64 { raw::ver_size(&self.ptr) }

	/// The uncompressed size of the .deb file.
	pub fn installed_size(&self) -> u64 { raw::ver_installed_size(&self.ptr) }

	/// The ID of the version.
	pub fn id(&self) -> u32 { raw::ver_id(&self.ptr) }

	/// If the version is able to be downloaded.
	pub fn downloadable(&self) -> bool { raw::ver_downloadable(&self.ptr) }

	/// Check if the version is installed
	pub fn is_installed(&self) -> bool { raw::ver_installed(&self.ptr) }

	/// Returns a reference to the Dependency Map owned by the Version
	/// ```
	/// let keys = [
	///    "Depends",
	///    "PreDepends",
	///    "Suggests",
	///    "Recommends",
	///    "Conflicts",
	///    "Replaces",
	///    "Obsoletes",
	///    "Breaks",
	///    "Enhances",
	/// ];
	/// ```
	/// Dependencies are in a `Vec<Dependency>`
	///
	/// The Dependency struct represents an Or Group of dependencies.
	/// The base deps are located in `Dependency.base_deps`
	///
	/// For example where we use the `"Depends"` key:
	///
	/// ```
	/// use rust_apt::cache::Cache;
	/// let cache = Cache::new();
	/// let version = cache.get("apt").unwrap().candidate().unwrap();
	/// for dep in version.depends_map().get("Depends").unwrap() {
	///    if dep.is_or() {
	///        for base_dep in &dep.base_deps {
	///            println!("{}", base_dep.name())
	///        }
	///    } else {
	///        // is_or is false so there is only one BaseDep
	///        println!("{}", dep.first().name())
	///    }
	/// }
	/// ```
	pub fn depends_map(&self) -> &HashMap<String, Vec<Dependency>> {
		self.depends_list.get_or_init(|| self.gen_depends())
	}

	/// Returns a reference Vector, if it exists, for the given key.
	///
	/// See the doc for `depends_map()` for more information.
	pub fn get_depends(&self, key: &str) -> Option<&Vec<Dependency>> {
		self.depends_list
			.get_or_init(|| self.gen_depends())
			.get(key)
	}

	/// Returns a Reference Vector, if it exists, for "Enhances".
	pub fn enhances(&self) -> Option<&Vec<Dependency>> { self.get_depends("Enhances") }

	/// Returns a Reference Vector, if it exists, for "Depends" and
	/// "PreDepends".
	pub fn dependencies(&self) -> Option<Vec<&Dependency>> {
		let mut ret_vec: Vec<&Dependency> = Vec::new();

		if let Some(dep_list) = self.get_depends("Depends") {
			for dep in dep_list {
				ret_vec.push(dep)
			}
		}
		if let Some(dep_list) = self.get_depends("PreDepends") {
			for dep in dep_list {
				ret_vec.push(dep)
			}
		}
		if ret_vec.is_empty() {
			return None;
		}
		Some(ret_vec)
	}

	/// Returns a Reference Vector, if it exists, for "Recommends".
	pub fn recommends(&self) -> Option<&Vec<Dependency>> { self.get_depends("Recommends") }

	/// Returns a Reference Vector, if it exists, for "suggests".
	pub fn suggests(&self) -> Option<&Vec<Dependency>> { self.get_depends("Suggests") }

	/// Get the translated long description
	pub fn description(&self) -> String {
		let mut records = self.records.borrow_mut();
		records.lookup_desc(&self.ptr.desc);
		records.description()
	}

	/// Get the translated short description
	pub fn summary(&self) -> String {
		let mut records = self.records.borrow_mut();
		records.lookup_desc(&self.ptr.desc);
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
		let ver_file = ver_file_list(&self.ptr).into_iter().next()?;
		let mut records = self.records.borrow_mut();

		records.lookup_ver(&ver_file);
		records.hash_find(hash_type)
	}

	/// Returns an iterator of PackageFiles (Origins) for the version
	pub fn package_files(&self) -> impl Iterator<Item = PackageFile> + '_ {
		ver_pkg_file_list(&self.ptr)
			.into_iter()
			.map(|pkg_file| PackageFile::new(pkg_file, Rc::clone(&self.records.borrow().cache)))
	}

	/// Returns an iterator of URIs for the version
	pub fn uris(&self) -> impl Iterator<Item = String> + '_ {
		ver_file_list(&self.ptr).into_iter().filter_map(|ver_file| {
			let mut records = self.records.borrow_mut();
			records.lookup_ver(&ver_file);

			let uri = records.uri(&ver_file);
			if !uri.starts_with("file:") {
				Some(uri)
			} else {
				None
			}
		})
	}

	/// Internal Method for converting raw::deps into rust-apt deps
	fn convert_depends(&self, raw_deps: raw::DepContainer) -> Dependency {
		let mut base_vec = Vec::new();
		for base_dep in raw_deps.dep_list {
			base_vec.push(BaseDep {
				records: Rc::clone(&self.records),
				depcache: Rc::clone(&self.depcache),
				apt_dep: base_dep,
			})
		}
		Dependency {
			dep_type: raw_deps.dep_type,
			base_deps: base_vec,
		}
	}

	/// Internal Method for Generating the Dependency HashMap
	fn gen_depends(&self) -> HashMap<String, Vec<Dependency>> {
		let mut dependencies: HashMap<String, Vec<Dependency>> = HashMap::new();
		for dep in raw::dep_list(&self.ptr) {
			if let Some(vec) = dependencies.get_mut(&dep.dep_type) {
				vec.push(self.convert_depends(dep))
			} else {
				dependencies.insert(dep.dep_type.to_owned(), vec![self.convert_depends(dep)]);
			}
		}
		dependencies
	}
}

impl<'a> fmt::Display for Version<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{}: Version {} <ID: {}, arch: {}, size: {}, installed_size: {}, section: {} Priority \
			 {} at {}, downloadable: {}>",
			self.parent().name(),
			self.version(),
			self.id(),
			self.arch(),
			unit_str(self.size(), NumSys::Decimal),
			unit_str(self.installed_size(), NumSys::Decimal),
			self.section(),
			self.priority_str(),
			self.priority(),
			self.downloadable(),
		)?;
		Ok(())
	}
}

// Implementations for comparing versions.
impl<'a> PartialEq for Version<'a> {
	fn eq(&self, other: &Self) -> bool {
		matches!(
			cmp_versions(&self.version(), &other.version()),
			Ordering::Equal
		)
	}
}

impl<'a> PartialOrd for Version<'a> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(cmp_versions(&self.version(), &other.version()))
	}
}

/// A struct representing a Base Dependency
#[derive(Debug)]
pub struct BaseDep {
	apt_dep: raw::BaseDep,
	records: Rc<RefCell<Records>>,
	depcache: Rc<RefCell<DepCache>>,
}

impl BaseDep {
	pub fn name(&self) -> &String { &self.apt_dep.name }

	pub fn version(&self) -> &String { &self.apt_dep.version }

	pub fn comp(&self) -> &String { &self.apt_dep.comp }

	pub fn dep_type(&self) -> &String { &self.apt_dep.dep_type }

	pub fn all_targets(&self) -> impl Iterator<Item = Version> {
		raw::dep_all_targets(&self.apt_dep)
			.into_iter()
			.map(|ver_ptr| {
				Version::new(Rc::clone(&self.records), Rc::clone(&self.depcache), ver_ptr)
			})
	}
}

impl fmt::Display for BaseDep {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"BaseDep <Name: {}, Version: {}, Comp: {}, Type: {}>",
			self.name(),
			self.version(),
			self.comp(),
			self.dep_type(),
		)?;
		Ok(())
	}
}

/// A struct representing an Or_Group of Dependencies
#[derive(Debug)]
pub struct Dependency {
	pub dep_type: String,
	pub base_deps: Vec<BaseDep>,
}

impl Dependency {
	/// Returns True if there are multiple dependencies that can satisfy this
	pub fn is_or(&self) -> bool { self.base_deps.len() > 1 }

	/// Returns a reference to the first BaseDep
	pub fn first(&self) -> &BaseDep { &self.base_deps[0] }
}

impl fmt::Display for Dependency {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.is_or() {
			write!(f, "Or Dependencies[")?;
		} else {
			write!(f, "Dependency[")?;
		}
		for dep in &self.base_deps {
			write!(
				f,
				"\n    BaseDep <Name: {}, Version: {}, Comp: {}, Type: {}>,",
				dep.name(),
				dep.version(),
				dep.comp(),
				dep.dep_type(),
			)?;
		}
		write!(f, "\n]")?;
		Ok(())
	}
}

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {
	/// Struct representing a base dependency.
	struct BaseDep {
		name: String,
		version: String,
		comp: String,
		dep_type: String,
		ptr: SharedPtr<DepIterator>,
	}

	/// A wrapper for the BaseDeps to be passed as a list across the barrier.
	struct DepContainer {
		dep_type: String,
		dep_list: Vec<BaseDep>,
	}

	unsafe extern "C++" {
		type DepIterator;

		type PkgCacheFile = crate::cache::raw::PkgCacheFile;
		type VersionPtr = crate::cache::raw::VersionPtr;
		type PackagePtr = crate::cache::raw::PackagePtr;
		type VersionFile = crate::cache::raw::VersionFile;

		include!("rust-apt/apt-pkg-c/cache.h");
		include!("rust-apt/apt-pkg-c/package.h");

		/// Return the installed version of the package.
		/// Ptr will be NULL if it's not installed.
		pub fn pkg_current_version(iterator: &PackagePtr) -> VersionPtr;

		/// Return the candidate version of the package.
		/// Ptr will be NULL if there isn't a candidate.
		pub fn pkg_candidate_version(
			cache: &UniquePtr<PkgCacheFile>,
			iterator: &PackagePtr,
		) -> VersionPtr;

		/// Check if the package is installed.
		pub fn pkg_is_installed(iterator: &PackagePtr) -> bool;

		/// Check if the package has versions.
		/// If a package has no versions it is considered virtual.
		pub fn pkg_has_versions(iterator: &PackagePtr) -> bool;

		/// Check if a package provides anything.
		/// Virtual packages may provide a real package.
		/// This is how you would access the packages to satisfy it.
		pub fn pkg_has_provides(iterator: &PackagePtr) -> bool;

		/// Return true if the package is essential, otherwise false.
		pub fn pkg_essential(iterator: &PackagePtr) -> bool;

		/// Get the fullname of a package.
		/// More information on this in the package module.
		pub fn get_fullname(iterator: &PackagePtr, pretty: bool) -> String;

		/// Get the name of the package without the architecture.
		pub fn pkg_name(pkg: &PackagePtr) -> String;

		/// Get the architecture of a package.
		pub fn pkg_arch(iterator: &PackagePtr) -> String;

		/// Get the ID of a package.
		pub fn pkg_id(iterator: &PackagePtr) -> u32;

		/// Get the current state of a package.
		pub fn pkg_current_state(iterator: &PackagePtr) -> u8;

		/// Get the installed state of a package.
		pub fn pkg_inst_state(iterator: &PackagePtr) -> u8;

		/// Get the selected state of a package.
		pub fn pkg_selected_state(iterator: &PackagePtr) -> u8;

		/// Version Functions:

		/// Return a Vector of all the dependencies of a version.
		pub fn dep_list(version: &VersionPtr) -> Vec<DepContainer>;

		/// Return the parent package.
		pub fn ver_parent(version: &VersionPtr) -> PackagePtr;

		/// The architecture of a version.
		pub fn ver_arch(version: &VersionPtr) -> String;

		/// The version string of the version. "1.4.10"
		pub fn ver_str(version: &VersionPtr) -> String;

		/// The section of the version as shown in `apt show`.
		pub fn ver_section(version: &VersionPtr) -> String;

		/// The priority string as shown in `apt show`.
		pub fn ver_priority_str(version: &VersionPtr) -> String;

		/// The name of the source package the version was built from.
		pub fn ver_source_name(version: &VersionPtr) -> String;

		/// The version of the source package.
		pub fn ver_source_version(version: &VersionPtr) -> String;

		/// The priority of the package as shown in `apt policy`.
		pub fn ver_priority(cache: &UniquePtr<PkgCacheFile>, version: &VersionPtr) -> i32;

		/// The size of the .deb file.
		pub fn ver_size(version: &VersionPtr) -> u64;

		/// The uncompressed size of the .deb file.
		pub fn ver_installed_size(version: &VersionPtr) -> u64;

		/// The ID of the version.
		pub fn ver_id(version: &VersionPtr) -> u32;

		/// If the version is able to be downloaded.
		pub fn ver_downloadable(version: &VersionPtr) -> bool;

		/// Check if the version is currently installed.
		pub fn ver_installed(version: &VersionPtr) -> bool;

		/// Dependency Functions:

		/// Return a Vector of all versions that can satisfy a dependency.
		pub fn dep_all_targets(dep: &BaseDep) -> Vec<VersionPtr>;
	}
}

impl fmt::Debug for raw::BaseDep {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"BaseDep <Name: {}, Version: {}, Comp: {}, Type: {}>",
			self.name, self.version, self.comp, self.dep_type,
		)?;
		Ok(())
	}
}
