//! Contains Package, Version and Dependency Structs.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::ops::Deref;

use once_cell::unsync::OnceCell;

use crate::cache::Cache;
use crate::raw::package::{RawDependency, RawPackage, RawPackageFile, RawProvider, RawVersion};
use crate::util::cmp_versions;

pub struct Package<'a> {
	ptr: RawPackage,
	cache: &'a Cache,
}

impl<'a> Package<'a> {
	pub fn new(cache: &'a Cache, ptr: RawPackage) -> Package<'a> { Package { ptr, cache } }

	/// Internal Method for generating the version list.
	fn raw_versions(&self) -> impl Iterator<Item = RawVersion> {
		self.version_list().into_iter().flatten()
	}

	/// Return either a Version or None
	///
	/// # Example:
	/// ```
	/// use rust_apt::new_cache;
	///
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("apt").unwrap();
	///
	/// pkg.get_version("2.4.7");
	/// ```
	pub fn get_version(&'a self, version_str: &str) -> Option<Version<'a>> {
		for ver in self.raw_versions() {
			if version_str == ver.version() {
				return Some(Version::new(ver, self));
			}
		}
		None
	}

	/// Returns the version object of the installed version.
	///
	/// If there isn't an installed version, returns None
	pub fn installed(&'a self) -> Option<Version<'a>> {
		// Cxx error here just indicates that the Version doesn't exist
		Some(Version::new(self.current_version()?, self))
	}

	/// Returns the version object of the candidate.
	///
	/// If there isn't a candidate, returns None
	pub fn candidate(&'a self) -> Option<Version<'a>> {
		// Cxx error here just indicates that the Version doesn't exist
		Some(Version::new(
			self.cache.depcache().candidate_version(self)?,
			self,
		))
	}

	/// Returns a version list
	/// starting with the newest and ending with the oldest.
	pub fn versions(&'a self) -> impl Iterator<Item = Version<'a>> {
		self.raw_versions().map(|ver| Version::new(ver, self))
	}

	/// Returns a list of providers
	pub fn provides(&'a self) -> impl Iterator<Item = Provider<'a>> {
		self.provides_list()
			.into_iter()
			.flatten()
			.map(|provider| Provider::new(provider, self.cache))
	}

	/// Check if the package is upgradable.
	///
	/// ## skip_depcache:
	///
	/// Skipping the DepCache is unnecessary if it's already been initialized.
	/// If you're unsure use `false`
	///
	///   * [true] = Increases performance by skipping the pkgDepCache.
	///   * [false] = Use DepCache to check if the package is upgradable
	pub fn is_upgradable(&self) -> bool {
		self.is_installed() && self.cache.depcache().is_upgradable(self)
	}

	/// Check if the package is auto installed. (Not installed by the user)
	pub fn is_auto_installed(&self) -> bool { self.cache.depcache().is_auto_installed(self) }

	/// Check if the package is auto removable
	pub fn is_auto_removable(&self) -> bool {
		(self.is_installed() || self.marked_install()) && self.cache.depcache().is_garbage(self)
	}

	/// Check if the package is now broken
	pub fn is_now_broken(&self) -> bool { self.cache.depcache().is_now_broken(self) }

	/// Check if the package package installed is broken
	pub fn is_inst_broken(&self) -> bool { self.cache.depcache().is_inst_broken(self) }

	/// Check if the package is marked install
	pub fn marked_install(&self) -> bool { self.cache.depcache().marked_install(self) }

	/// Check if the package is marked upgrade
	pub fn marked_upgrade(&self) -> bool { self.cache.depcache().marked_upgrade(self) }

	/// Check if the package is marked purge
	pub fn marked_purge(&self) -> bool { self.cache.depcache().marked_purge(self) }

	/// Check if the package is marked delete
	pub fn marked_delete(&self) -> bool { self.cache.depcache().marked_delete(self) }

	/// Check if the package is marked keep
	pub fn marked_keep(&self) -> bool { self.cache.depcache().marked_keep(self) }

	/// Check if the package is marked downgrade
	pub fn marked_downgrade(&self) -> bool { self.cache.depcache().marked_downgrade(self) }

	/// Check if the package is marked reinstall
	pub fn marked_reinstall(&self) -> bool { self.cache.depcache().marked_reinstall(self) }

	/// # Mark a package as automatically installed.
	///
	/// ## mark_auto:
	///   * [true] = Mark the package as automatically installed.
	///   * [false] = Mark the package as manually installed.
	pub fn mark_auto(&self, mark_auto: bool) -> bool {
		self.cache.depcache().mark_auto(self, mark_auto);
		// Convert to a bool to remain consistent with other mark functions.
		true
	}

	/// # Mark a package for keep.
	///
	/// ## Returns:
	///   * [true] if the mark was successful
	///   * [false] if the mark was unsuccessful
	///
	/// This means that the package will not be changed from its current
	/// version. This will not stop a reinstall, but will stop removal, upgrades
	/// and downgrades
	///
	/// We don't believe that there is any reason to unmark packages for keep.
	/// If someone has a reason, and would like it implemented, please put in a
	/// feature request.
	pub fn mark_keep(&self) -> bool { self.cache.depcache().mark_keep(self) }

	/// # Mark a package for removal.
	///
	/// ## Returns:
	///   * [true] if the mark was successful
	///   * [false] if the mark was unsuccessful
	///
	/// ## purge:
	///   * [true] = Configuration files will be removed along with the package.
	///   * [false] = Only the package will be removed.
	pub fn mark_delete(&self, purge: bool) -> bool {
		self.cache.depcache().mark_delete(self, purge)
	}

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
	pub fn mark_install(&self, auto_inst: bool, from_user: bool) -> bool {
		self.cache
			.depcache()
			.mark_install(self, auto_inst, from_user)
	}

	/// # Mark a package for reinstallation.
	///
	/// ## Returns:
	///   * [true] if the mark was successful
	///   * [false] if the mark was unsuccessful
	///
	/// ## reinstall:
	///   * [true] = The package will be marked for reinstall.
	///   * [false] = The package will be unmarked for reinstall.
	pub fn mark_reinstall(&self, reinstall: bool) -> bool {
		self.cache.depcache().mark_reinstall(self, reinstall);
		// Convert to a bool to remain consistent with other mark functions/
		true
	}

	/// Protect a package's state
	/// for when [`crate::cache::Cache::resolve`] is called.
	pub fn protect(&self) { self.cache.resolver().protect(self) }
}

impl<'a> Deref for Package<'a> {
	type Target = RawPackage;

	#[inline]
	fn deref(&self) -> &RawPackage { &self.ptr }
}

// Implementations for comparing packages.
impl<'a> PartialEq for Package<'a> {
	fn eq(&self, other: &Self) -> bool { self.id() == other.id() }
}

pub struct Version<'a> {
	ptr: RawVersion,
	parent: &'a Package<'a>,
	cache: &'a Cache,
	depends_map: OnceCell<HashMap<DepType, Vec<Dependency<'a, 'a>>>>,
}

impl<'a> Version<'a> {
	pub fn new(ptr: RawVersion, parent: &'a Package) -> Version<'a> {
		Version {
			ptr,
			parent,
			cache: parent.cache,
			depends_map: OnceCell::new(),
		}
	}

	/// Returns a list of providers
	pub fn provides(&'a self) -> impl Iterator<Item = Provider<'a>> {
		self.provides_list()
			.into_iter()
			.flatten()
			.map(|provider| Provider::new(provider, self.cache))
	}

	/// Returns an iterator of PackageFiles (Origins) for the version
	pub fn package_files(&self) -> impl Iterator<Item = RawPackageFile> + '_ {
		// TODO: We should probably not expect here.
		self.version_files()
			.expect("No Version Files")
			.map(|pkg_file| pkg_file.pkg_file())
	}

	/// Return the version's parent package.
	pub fn parent(&'a self) -> &'a Package<'a> { self.parent }

	/// Returns a reference to the Dependency Map owned by the Version
	///
	/// Dependencies are in a `Vec<Dependency>`
	///
	/// The Dependency struct represents an Or Group of dependencies.
	/// The base deps are located in `Dependency.base_deps`
	///
	/// For example where we use the `DepType::Depends` key:
	///
	/// ```
	/// use rust_apt::new_cache;
	/// use rust_apt::package::DepType;
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("apt").unwrap();
	/// let version = pkg.candidate().unwrap();
	/// for dep in version.depends_map().get(&DepType::Depends).unwrap() {
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
	pub fn depends_map(&self) -> &HashMap<DepType, Vec<Dependency>> {
		self.depends_map.get_or_init(|| {
			let dep = self.depends().expect("Dependency was null");
			let mut dependencies: HashMap<DepType, Vec<Dependency>> = HashMap::new();

			while !dep.end() {
				let mut or_deps = vec![];
				or_deps.push(BaseDep::new(dep.unique(), self.parent));

				// This means that more than one thing can satisfy a dependency.
				if dep.compare_op() {
					loop {
						dep.raw_next();
						or_deps.push(BaseDep::new(dep.unique(), self.parent));
						// This is the last of the Or group
						if !dep.compare_op() {
							break;
						}
					}
				}

				let dep_type = DepType::from(dep.dep_type());

				// If the entry already exists in the map append it.
				if let Some(vec) = dependencies.get_mut(&dep_type) {
					vec.push(Dependency { base_deps: or_deps })
				} else {
					// Doesn't exist so we create it
					dependencies.insert(dep_type, vec![Dependency { base_deps: or_deps }]);
				}

				dep.raw_next();
			}
			dbg!(&dependencies);
			dependencies
		})
	}

	/// Returns a reference Vector, if it exists, for the given key.
	///
	/// See the doc for `depends_map()` for more information.
	pub fn get_depends(&'a self, key: &DepType) -> Option<&Vec<Dependency>> {
		self.depends_map().get(key)
	}

	/// Returns a Reference Vector, if it exists, for "Enhances".
	pub fn enhances(&'a self) -> Option<&Vec<Dependency>> { self.get_depends(&DepType::Enhances) }

	/// Returns a Reference Vector, if it exists,
	/// for "Depends" and "PreDepends".
	pub fn dependencies(&'a self) -> Option<Vec<&Dependency>> {
		let mut ret_vec: Vec<&Dependency> = Vec::new();

		if let Some(dep_list) = self.get_depends(&DepType::Depends) {
			for dep in dep_list {
				ret_vec.push(dep)
			}
		}
		if let Some(dep_list) = self.get_depends(&DepType::PreDepends) {
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
	pub fn recommends(&'a self) -> Option<&Vec<Dependency>> {
		self.get_depends(&DepType::Recommends)
	}

	/// Returns a Reference Vector, if it exists, for "suggests".
	pub fn suggests(&'a self) -> Option<&Vec<Dependency>> { self.get_depends(&DepType::Suggests) }

	/// Get the translated long description
	pub fn description(&self) -> Option<String> {
		if let Some(desc_file) = self.description_files()?.next() {
			self.cache.records().desc_file_lookup(&desc_file);
			return self.cache.records().long_desc().ok();
		}
		None
	}

	/// Get the translated short description
	pub fn summary(&self) -> Option<String> {
		if let Some(desc_file) = self.description_files()?.next() {
			self.cache.records().desc_file_lookup(&desc_file);
			return self.cache.records().short_desc().ok();
		}
		None
	}

	/// Get data from the specified record field
	///
	/// # Returns:
	///   * Some String or None if the field doesn't exist.
	///
	/// # Example:
	/// ```
	/// use rust_apt::new_cache;
	/// use rust_apt::records::RecordField;
	///
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("apt").unwrap();
	/// let cand = pkg.candidate().unwrap();
	///
	/// println!("{}", cand.get_record(RecordField::Maintainer).unwrap());
	/// // Or alternatively you can just pass any string
	/// println!("{}", cand.get_record("Description-md5").unwrap());
	/// ```
	pub fn get_record<T: ToString + ?Sized>(&self, field: &T) -> Option<String> {
		if let Some(ver_file) = self.version_files()?.next() {
			self.cache.records().ver_file_lookup(&ver_file);
			return self.cache.records().get_field(field.to_string()).ok();
		}
		None
	}

	/// Get the hash specified. If there isn't one returns None
	/// `version.hash("md5sum")`
	pub fn hash<T: ToString + ?Sized>(&self, hash_type: &T) -> Option<String> {
		if let Some(ver_file) = self.version_files()?.next() {
			self.cache.records().ver_file_lookup(&ver_file);
			return self.cache.records().hash_find(hash_type.to_string()).ok();
		}
		None
	}

	/// Get the sha256 hash. If there isn't one returns None
	/// This is equivalent to `version.hash("sha256")`
	pub fn sha256(&self) -> Option<String> { self.hash("sha256") }

	/// Get the sha512 hash. If there isn't one returns None
	/// This is equivalent to `version.hash("sha512")`
	pub fn sha512(&self) -> Option<String> { self.hash("sha512") }

	/// Returns an iterator of URIs for the version
	pub fn uris(&'a self) -> impl Iterator<Item = String> + '_ {
		// TODO: Maybe remove Package_files method and make a map of ver_file pkg_file?
		self.package_files().filter_map(|mut pkg_file| {
			self.cache.find_index(&mut pkg_file);
			let ver_file = self.version_files()?.next()?;

			self.cache.records().ver_file_lookup(&ver_file);

			if let Ok(uri) = self.cache.records().ver_uri(&pkg_file) {
				// Should match this from the configurations. Hardcoding is okay for now.
				if !uri.ends_with("/var/lib/dpkg/status") {
					return Some(uri);
				}
			}
			None
		})
	}

	/// Set this version as the candidate.
	pub fn set_candidate(&self) { self.cache.depcache().set_candidate_version(self); }

	/// The priority of the Version as shown in `apt policy`.
	pub fn priority(&self) -> i32 { self.cache.priority(self) }
}

// Implementations for comparing versions.
impl<'a> PartialEq for Version<'a> {
	fn eq(&self, other: &Self) -> bool {
		matches!(
			cmp_versions(self.version(), other.version()),
			Ordering::Equal
		)
	}
}

impl<'a> PartialOrd for Version<'a> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(cmp_versions(self.version(), other.version()))
	}
}

impl<'a> Deref for Version<'a> {
	type Target = RawVersion;

	#[inline]
	fn deref(&self) -> &RawVersion { &self.ptr }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum DepType {
	Depends,
	PreDepends,
	Suggests,
	Recommends,
	Conflicts,
	Replaces,
	Obsoletes,
	Breaks,
	Enhances,
}

impl From<u8> for DepType {
	fn from(value: u8) -> Self {
		match value {
			1 => DepType::Depends,
			2 => DepType::PreDepends,
			3 => DepType::Suggests,
			4 => DepType::Recommends,
			5 => DepType::Conflicts,
			6 => DepType::Replaces,
			7 => DepType::Obsoletes,
			8 => DepType::Breaks,
			9 => DepType::Enhances,
			_ => panic!("Dependency is malformed?"),
		}
	}
}

/// A struct representing a Base Dependency.
pub struct BaseDep<'a, 'b> {
	ptr: RawDependency,
	/// Reference to the Package this dependency belongs too.
	pub parent: &'b Package<'a>,
}

impl<'a, 'b> BaseDep<'a, 'b> {
	pub fn new(ptr: RawDependency, parent: &'b Package<'a>) -> BaseDep<'a, 'b> {
		BaseDep { ptr, parent }
	}

	/// This is the name of the dependency.
	pub fn name(&self) -> String { self.target_pkg().name().to_string() }

	/// The version of the dependency if specified.
	pub fn version(&self) -> Option<&str> { self.target_ver().ok() }

	/// Comparison type of the dependency version, if specified.
	pub fn comp(&self) -> Option<&str> { self.comp_type().ok() }

	/// Iterate all Versions that are able to satisfy this dependency
	pub fn all_targets(&self) -> impl Iterator<Item = Version> {
		self.ptr
			.all_targets()
			.map(|ver| Version::new(ver, self.parent))
	}
}

impl<'a, 'b> Deref for BaseDep<'a, 'b> {
	type Target = RawDependency;

	#[inline]
	fn deref(&self) -> &RawDependency { &self.ptr }
}

impl<'a, 'b> Debug for BaseDep<'a, 'b> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"BaseDep <Name: '{}', Version: '{}', Comp: '{}', Type: '{}'>",
			self.target_pkg().name(),
			self.version().unwrap_or("None"),
			self.comp().unwrap_or("None"),
			self.dep_type(),
		)?;
		Ok(())
	}
}

/// A struct representing an Or_Group of Dependencies.
#[derive(Debug)]
pub struct Dependency<'a, 'b> {
	/// Vector of BaseDeps that can satisfy this dependency.
	pub base_deps: Vec<BaseDep<'a, 'b>>,
}

impl<'a, 'b> Dependency<'a, 'b> {
	/// Return the Dep Type of this group. Depends, Pre-Depends.
	pub fn dep_type(&self) -> DepType { DepType::from(self.base_deps[0].dep_type()) }

	/// Returns True if there are multiple dependencies that can satisfy this
	pub fn is_or(&self) -> bool { self.base_deps.len() > 1 }

	/// Returns a reference to the first BaseDep
	pub fn first(&self) -> &BaseDep { &self.base_deps[0] }
}

pub struct Provider<'a> {
	ptr: RawProvider,
	cache: &'a Cache,
	target_pkg: Package<'a>,
}

impl<'a> Provider<'a> {
	pub fn new(ptr: RawProvider, cache: &'a Cache) -> Provider<'a> {
		let target_pkg = Package::new(cache, ptr.target_pkg());
		Provider {
			ptr,
			cache,
			target_pkg,
		}
	}

	/// Return the Target Package of the provider.
	pub fn package(&self) -> Package<'a> { Package::new(self.cache, self.target_pkg()) }

	/// Return the Target Version of the provider.
	pub fn version(&'a self) -> Version<'a> { Version::new(self.target_ver(), &self.target_pkg) }
}

impl<'a> Deref for Provider<'a> {
	type Target = RawProvider;

	#[inline]
	fn deref(&self) -> &RawProvider { &self.ptr }
}
