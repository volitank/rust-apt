//! Contains Package, Version and Dependency Structs.

use std::cell::OnceCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::cache::Cache;
use crate::raw::package::{RawDependency, RawPackage, RawPackageFile, RawProvider, RawVersion};
use crate::util::cmp_versions;

pub struct Package<'a> {
	ptr: RawPackage,
	pub(crate) cache: &'a Cache,
	rdepends_map: OnceCell<HashMap<DepType, Vec<Dependency<'a>>>>,
}

impl<'a> Package<'a> {
	pub fn new(cache: &'a Cache, ptr: RawPackage) -> Package<'a> {
		Package {
			ptr,
			cache,
			rdepends_map: OnceCell::new(),
		}
	}

	/// Internal Method for generating the version list.
	fn raw_versions(&self) -> impl Iterator<Item = RawVersion> {
		self.version_list().into_iter().flatten()
	}

	/// Returns a Reverse Dependency Map of the package
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
	/// for dep in pkg.rdepends_map().get(&DepType::Depends).unwrap() {
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
	pub fn rdepends_map(&self) -> &HashMap<DepType, Vec<Dependency<'a>>> {
		self.rdepends_map
			.get_or_init(|| create_depends_map(self.cache, self.rev_depends_list()))
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
				return Some(Version::new(ver, self.cache));
			}
		}
		None
	}

	/// Returns the version object of the installed version.
	///
	/// If there isn't an installed version, returns None
	pub fn installed(&self) -> Option<Version> {
		// Cxx error here just indicates that the Version doesn't exist
		Some(Version::new(self.current_version()?, self.cache))
	}

	/// Returns the version object of the candidate.
	///
	/// If there isn't a candidate, returns None
	pub fn candidate(&self) -> Option<Version> {
		// Cxx error here just indicates that the Version doesn't exist
		Some(Version::new(
			self.cache.depcache().candidate_version(self)?,
			self.cache,
		))
	}

	/// Returns a version list
	/// starting with the newest and ending with the oldest.
	pub fn versions(&self) -> impl Iterator<Item = Version> {
		self.raw_versions().map(|ver| Version::new(ver, self.cache))
	}

	/// Returns a list of providers
	pub fn provides(&self) -> impl Iterator<Item = Provider<'a>> {
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

impl<'a> fmt::Display for Package<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.name())?;
		Ok(())
	}
}

impl<'a> fmt::Debug for Package<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let versions: Vec<Version> = self.versions().collect();
		f.debug_struct("Package")
			.field("name", &self.name())
			.field("arch", &self.arch())
			.field("virtual", &versions.is_empty())
			.field("versions", &versions)
			.finish_non_exhaustive()
	}
}

pub struct Version<'a> {
	ptr: RawVersion,
	cache: &'a Cache,
	depends_map: OnceCell<HashMap<DepType, Vec<Dependency<'a>>>>,
}

impl<'a> Version<'a> {
	pub fn new(ptr: RawVersion, cache: &'a Cache) -> Version<'a> {
		Version {
			ptr,
			cache,
			depends_map: OnceCell::new(),
		}
	}

	/// Returns a list of providers
	pub fn provides(&self) -> impl Iterator<Item = Provider<'a>> {
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
	pub fn parent(&self) -> Package<'a> { Package::new(self.cache, self.parent_pkg()) }

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
	pub fn depends_map(&self) -> &HashMap<DepType, Vec<Dependency<'a>>> {
		self.depends_map
			.get_or_init(|| create_depends_map(self.cache, self.depends()))
	}

	/// Returns a reference Vector, if it exists, for the given key.
	///
	/// See the doc for `depends_map()` for more information.
	pub fn get_depends(&self, key: &DepType) -> Option<&Vec<Dependency<'a>>> {
		self.depends_map().get(key)
	}

	/// Returns a Reference Vector, if it exists, for "Enhances".
	pub fn enhances(&self) -> Option<&Vec<Dependency<'a>>> { self.get_depends(&DepType::Enhances) }

	/// Returns a Reference Vector, if it exists,
	/// for "Depends" and "PreDepends".
	pub fn dependencies(&self) -> Option<Vec<&Dependency<'a>>> {
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
	pub fn recommends(&self) -> Option<&Vec<Dependency<'a>>> {
		self.get_depends(&DepType::Recommends)
	}

	/// Returns a Reference Vector, if it exists, for "suggests".
	pub fn suggests(&self) -> Option<&Vec<Dependency<'a>>> { self.get_depends(&DepType::Suggests) }

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

impl<'a> fmt::Display for Version<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.version())?;
		Ok(())
	}
}

impl<'a> fmt::Debug for Version<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let parent = self.parent();
		// Lifetimes make us have to do some weird things.
		let is_candidate = match self.cache.depcache().candidate_version(&parent) {
			Some(cand) => {
				let temp_ver = Version::new(cand, self.cache);
				self == &temp_ver
			},
			None => false,
		};

		f.debug_struct("Version")
			.field("pkg", &parent.name())
			.field("arch", &self.arch())
			.field("version", &self.version())
			.field("is_candidate", &is_candidate)
			.field("is_installed", &self.is_installed())
			.finish_non_exhaustive()
	}
}

pub fn create_depends_map(
	cache: &Cache,
	dep: Option<RawDependency>,
) -> HashMap<DepType, Vec<Dependency>> {
	let mut dependencies: HashMap<DepType, Vec<Dependency>> = HashMap::new();

	if let Some(dep) = dep {
		while !dep.end() {
			let mut or_deps = vec![];
			or_deps.push(BaseDep::new(dep.unique(), cache));

			// This means that more than one thing can satisfy a dependency.
			// For reverse dependencies we cannot get the or deps.
			// This can cause a segfault
			// See: https://gitlab.com/volian/rust-apt/-/merge_requests/36
			if dep.compare_op() && !dep.is_reverse() {
				loop {
					dep.raw_next();
					or_deps.push(BaseDep::new(dep.unique(), cache));
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
	}
	dependencies
}

#[derive(fmt::Debug, Eq, PartialEq, Hash)]
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

impl AsRef<str> for DepType {
	fn as_ref(&self) -> &str {
		match self {
			DepType::Depends => "Depends",
			DepType::PreDepends => "PreDepends",
			DepType::Suggests => "Suggests",
			DepType::Recommends => "Recommends",
			DepType::Conflicts => "Conflicts",
			DepType::Replaces => "Replaces",
			DepType::Obsoletes => "Obsoletes",
			DepType::Breaks => "Breaks",
			DepType::Enhances => "Enhances",
		}
	}
}

/// A struct representing a Base Dependency.
pub struct BaseDep<'a> {
	ptr: RawDependency,
	cache: &'a Cache,
	target: OnceCell<Package<'a>>,
	parent_ver: OnceCell<RawVersion>,
}

impl<'a> BaseDep<'a> {
	pub fn new(ptr: RawDependency, cache: &'a Cache) -> BaseDep {
		BaseDep {
			ptr,
			cache,
			target: OnceCell::new(),
			parent_ver: OnceCell::new(),
		}
	}

	/// This is the name of the dependency.
	pub fn name(&self) -> &str { self.target_package().name() }

	/// Return the target package.
	///
	/// For Reverse Dependencies this will actually return the parent package
	pub fn target_package(&self) -> &Package<'a> {
		self.target.get_or_init(|| {
			if self.is_reverse() {
				Package::new(self.cache, self.parent_pkg())
			} else {
				Package::new(self.cache, self.target_pkg())
			}
		})
	}

	/// The target version &str of the dependency if specified.
	pub fn version(&self) -> Option<&str> {
		if self.is_reverse() {
			Some(self.parent_ver.get_or_init(|| self.parent_ver()).version())
		} else {
			self.target_ver().ok()
		}
	}

	/// Comparison type of the dependency version, if specified.
	pub fn comp(&self) -> Option<&str> { self.comp_type().ok() }

	// Iterate all Versions that are able to satisfy this dependency
	pub fn all_targets(&self) -> impl Iterator<Item = RawVersion> { self.ptr.all_targets() }
}

impl<'a> Deref for BaseDep<'a> {
	type Target = RawDependency;

	#[inline]
	fn deref(&self) -> &RawDependency { &self.ptr }
}

impl<'a> fmt::Display for BaseDep<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if let (Some(comp), Some(version)) = (self.comp(), self.version()) {
			write!(f, "({} {comp} {version})", self.name(),)
		} else {
			write!(f, "({})", self.name(),)
		}
	}
}

impl<'a> fmt::Debug for BaseDep<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("BaseDep")
			.field("parent", &self.parent_pkg().name())
			.field("name", &self.name())
			.field("comp", &self.comp())
			.field("version", &self.version())
			.field("dep_type", &DepType::from(self.dep_type()))
			.field("is_reverse", &self.is_reverse())
			.finish()
	}
}

/// A struct representing an Or_Group of Dependencies.
#[derive(fmt::Debug)]
pub struct Dependency<'a> {
	/// Vector of BaseDeps that can satisfy this dependency.
	pub base_deps: Vec<BaseDep<'a>>,
}

impl<'a> Dependency<'a> {
	/// Return the Dep Type of this group. Depends, Pre-Depends.
	pub fn dep_type(&self) -> DepType { DepType::from(self.base_deps[0].dep_type()) }

	/// Returns True if there are multiple dependencies that can satisfy this
	pub fn is_or(&self) -> bool { self.base_deps.len() > 1 }

	/// Returns a reference to the first BaseDep
	pub fn first(&self) -> &BaseDep<'a> { &self.base_deps[0] }
}

impl<'a> fmt::Display for Dependency<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut dep_str = String::new();

		for (i, base_dep) in self.base_deps.iter().enumerate() {
			dep_str += &base_dep.to_string();
			if i + 1 != self.base_deps.len() {
				dep_str += " | "
			}
		}

		write!(
			f,
			"{} {:?} {dep_str}",
			self.first().parent_pkg().fullname(false),
			self.dep_type(),
		)?;
		Ok(())
	}
}

pub struct Provider<'a> {
	ptr: RawProvider,
	cache: &'a Cache,
}

impl<'a> Provider<'a> {
	pub fn new(ptr: RawProvider, cache: &'a Cache) -> Provider<'a> { Provider { ptr, cache } }

	/// Return the Target Package of the provider.
	pub fn package(&self) -> Package<'a> { Package::new(self.cache, self.target_pkg()) }

	/// Return the Target Version of the provider.
	pub fn version(&'a self) -> Version<'a> { Version::new(self.target_ver(), self.cache) }
}

impl<'a> Deref for Provider<'a> {
	type Target = RawProvider;

	#[inline]
	fn deref(&self) -> &RawProvider { &self.ptr }
}

impl<'a> fmt::Display for Provider<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let version = self.version();
		write!(
			f,
			"{} provides {} {}",
			self.name(),
			version.parent().fullname(false),
			version.version(),
		)?;
		Ok(())
	}
}

impl<'a> fmt::Debug for Provider<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Provider")
			.field("name", &self.name())
			.field("version", &self.version())
			.finish()
	}
}

// Implementation allowing structs to be put into a hashmap
impl<'a> Hash for Package<'a> {
	fn hash<H: Hasher>(&self, state: &mut H) { self.id().hash(state); }
}

impl<'a> Hash for Version<'a> {
	fn hash<H: Hasher>(&self, state: &mut H) { self.id().hash(state); }
}

impl<'a> Eq for Package<'a> {}
impl<'a> Eq for Version<'a> {}
