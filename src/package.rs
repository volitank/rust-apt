//! Contains Package, Version and Dependency Structs.
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use cxx::UniquePtr;
use once_cell::unsync::OnceCell;

use crate::cache::raw::{
	pkg_cache_find_name, pkg_list, pkg_version_list, ver_file_list, ver_pkg_file_list,
};
use crate::cache::{Cache, PackageFile, PackageSort, PointerMap};
use crate::depcache::DepCache;
use crate::records::Records;
use crate::resolver::ProblemResolver;
use crate::util::{cmp_versions, unit_str, NumSys};

/// Provide which Mark you want to apply on a package.
#[derive(Debug)]
pub enum Mark {
	/// Mark the package for keep.
	Keep,
	/// Mark the package as automatically installed.
	Auto,
	/// Mark the package as manually installed.
	Manual,
	/// Mark the package for removal.
	Remove,
	/// Mark the package to remove it and it's configuration files.
	Purge,
	/// Mark the package for install and mark it manually installed.
	///
	/// This includes the packages dependencies.
	/// Use the `mark_install` method on the package for finer control.
	Install,
	/// Mark a package for reinstall.
	Reinstall,
	/// Unmark a package for reinstall.
	///
	/// When getting the value this is always inverse of ReInstall.
	NoReinstall,
	/// Check if the package is marked for downgrade.
	///
	/// This currently does not work with `pkg.set()`. It will panic.
	/// This ability may be implemented in the future.
	Downgrade,
	/// Check if the package is marked as upgradable
	///
	/// /// When Setting a value this works the same as InstallWithDeps
	Upgrade,
}

/// A struct representing an `apt` Package
#[derive(Debug)]
pub struct Package<'a> {
	_lifetime: &'a PhantomData<Cache>,
	records: Rc<RefCell<Records>>,
	cache_ptr: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
	depcache: Rc<RefCell<DepCache>>,
	resolver: Rc<RefCell<ProblemResolver>>,
	pointer_map: Rc<RefCell<PointerMap>>,
	pub(crate) ptr: Rc<RefCell<raw::PackagePtr>>,
}

impl<'a> Package<'a> {
	pub(crate) fn new(
		records: Rc<RefCell<Records>>,
		cache_ptr: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
		depcache: Rc<RefCell<DepCache>>,
		resolver: Rc<RefCell<ProblemResolver>>,
		pointer_map: Rc<RefCell<PointerMap>>,
		ptr: Rc<RefCell<raw::PackagePtr>>,
	) -> Package<'a> {
		Package {
			_lifetime: &PhantomData,
			records,
			cache_ptr,
			depcache,
			resolver,
			pointer_map,
			ptr,
		}
	}

	// Internal method to create a package from itself
	pub(crate) fn make_package(&self, new_ptr: raw::PackagePtr) -> Package {
		Package::new(
			Rc::clone(&self.records),
			Rc::clone(&self.cache_ptr),
			Rc::clone(&self.depcache),
			Rc::clone(&self.resolver),
			Rc::clone(&self.pointer_map),
			Rc::clone(&self.pointer_map.borrow_mut().get_package(new_ptr)),
		)
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
	pub fn fullname(&self, pretty: bool) -> String { raw::get_fullname(&self.ptr.borrow(), pretty) }

	/// Return the name of the package without the architecture
	pub fn name(&self) -> String { raw::pkg_name(&self.ptr.borrow()) }

	/// Get the architecture of the package.
	pub fn arch(&self) -> String { raw::pkg_arch(&self.ptr.borrow()) }

	/// Get the ID of the package.
	pub fn id(&self) -> u32 { raw::pkg_id(&self.ptr.borrow()) }

	/// The current state of the package.
	pub fn current_state(&self) -> u8 { raw::pkg_current_state(&self.ptr.borrow()) }

	/// The installed state of the package.
	pub fn inst_state(&self) -> u8 { raw::pkg_inst_state(&self.ptr.borrow()) }

	/// The selected state of the package.
	pub fn selected_state(&self) -> u8 { raw::pkg_selected_state(&self.ptr.borrow()) }

	/// Check if the package is essential or not.
	pub fn essential(&self) -> bool { raw::pkg_essential(&self.ptr.borrow()) }

	/// Check if the package has versions.
	pub fn has_versions(&self) -> bool { raw::pkg_has_versions(&self.ptr.borrow()) }

	/// Check if the package has provides.
	pub fn has_provides(&self) -> bool { raw::pkg_has_provides(&self.ptr.borrow()) }

	/// The list of packages that provide this package.
	///
	/// # version_rel:
	/// * [Some] = A tuple representing the version and operator (i.e.
	///   `("1.0.0", ">=")`).
	/// * [None] = No version/operator qualifier will be used.
	///
	/// If an invalid operator is passed when using the [`Some`] variant
	/// (anything other than `<<`, `<=`, `=`, `>=`, and `>>`), than this
	/// function will panic.
	///
	/// This function searches the cache for any packages that provide this
	/// package. It is meant to get the list of packages that satisfy a
	/// dependency string. Take this example from a control file:
	/// ```text
	/// \# Example 1:
	/// Depends: apt-transport-https (= 5)
	/// \# Example 2:
	/// Depends: apt-transport-https
	/// ```
	///
	/// If `version_rel` is [`None`], then any packages that contain `Provides: apt-transport-https` will be unable to satisfy example 1, as per [Debian policy](https://www.debian.org/doc/debian-policy/ch-relationships.html#virtual-packages-provides).
	/// If `version_rel` is [`Some`] on the other hand, then this function will
	/// make sure any packages matching the provides entry (i.e. `Provides:
	/// apt-transport-https (= 5)`) fit into the version requirement. E.g. if
	/// `(">=", "6.0.0")` was provided, then the previous example of `Provides:
	/// apt-transport-https (= 5)` would not be returned, since it didn't fit
	/// the requirements of being greater than or equal to `6.0.0`.
	///
	/// # Returns:
	/// A vector of matching [`Version`] structs.
	///
	/// If you need to see what packages this package provides for, use a
	/// version from [`Package::candidate`] or [`Package::versions`] and call
	/// [`Version::provides_list`] instead.
	///
	/// NOTE: This function is currently fairly expensive to run (~300
	/// milliseconds on an Intel i5), and may dramatically slow down your
	/// program if called too much.
	pub fn rev_provides_list(&self, version_rel: Option<(&str, &str)>) -> Vec<Version> {
		// Make sure the passed in operator, if any, is valid.
		if let Some((operator, _)) = version_rel {
			if !vec!["<<", "<=", "=", ">=", ">>"].contains(&operator) {
				panic!("Invalid operator `{}`.", operator);
			}
		}

		let mut returned_pkgs = Vec::new();
		let master_pkgname = self.name();

		for pkg_ptr in pkg_list(&self.cache_ptr.borrow(), &PackageSort::default()) {
			let pkg = self.make_package(pkg_ptr);

			let pkg_pkgname = pkg.fullname(true);

			for version in pkg.versions() {
				let provides_pkgs = version.provides_list();

				for (provides_pkgname, provides_version) in provides_pkgs {
					// If the current package isn't meant to provide our package, it can never
					// match.
					#[allow(clippy::if_same_then_else)]
					if provides_pkgname != master_pkgname {
						continue;
					// If the client passed in a version_rel and this has no
					// version, it can never match.
					} else if version_rel.is_some() && provides_version.is_none() {
						continue;
					// If the client passed in no version_rel, it doesn't matter
					// what version this package provides, it'll always match.
					} else if version_rel.is_none() {
						let ret_ptr =
							pkg_cache_find_name(&self.cache_ptr.borrow(), pkg_pkgname.to_owned())
								.unwrap();
						returned_pkgs.push(
							self.make_package(ret_ptr)
								.get_version(&version.version())
								.unwrap(),
						);
						continue;
					}

					// Make sure the version_rel is good.
					let (version_operator, ver_version) = version_rel.unwrap();
					let ver_cmp_result =
						cmp_versions(ver_version, provides_version.as_ref().unwrap());

					let good_version = match version_operator {
						"<<" => ver_cmp_result == Ordering::Less,
						"<=" => vec![Ordering::Less, Ordering::Equal].contains(&ver_cmp_result),
						"=" => ver_cmp_result == Ordering::Equal,
						">=" => vec![Ordering::Equal, Ordering::Greater].contains(&ver_cmp_result),
						">>" => ver_cmp_result == Ordering::Greater,
						_ => unreachable!(),
					};

					if good_version {
						let ret_ptr =
							pkg_cache_find_name(&self.cache_ptr.borrow(), pkg_pkgname.to_owned())
								.unwrap();
						returned_pkgs.push(
							self.make_package(ret_ptr)
								.get_version(&version.version())
								.unwrap(),
						);
					}
				}
			}
		}
		returned_pkgs
	}

	/// Internal method for creating a version
	fn create_version(&self, ver: raw::VersionPtr) -> Version<'a> {
		Version::new(
			Rc::clone(&self.records),
			Rc::clone(&self.cache_ptr),
			Rc::clone(&self.depcache),
			Rc::clone(&self.resolver),
			Rc::clone(&self.pointer_map),
			Rc::clone(&self.ptr),
			self.pointer_map.borrow_mut().get_version(ver),
		)
	}

	/// Returns the version object of the candidate.
	///
	/// If there isn't a candidate, returns None
	pub fn candidate(&self) -> Option<Version<'a>> {
		match raw::pkg_candidate_version(&self.records.borrow().cache.borrow(), &self.ptr.borrow())
		{
			Ok(ver_ptr) => Some(self.create_version(ver_ptr)),
			// The error here is just that the version doesn't exist
			// A cxx quirk we're using to make returning Option easier
			Err(_) => None,
		}
	}

	/// Returns the version object of the installed version.
	///
	/// If there isn't an installed version, returns None
	pub fn installed(&self) -> Option<Version<'a>> {
		match raw::pkg_current_version(&self.ptr.borrow()) {
			Ok(ver_ptr) => Some(self.create_version(ver_ptr)),
			// The error here is just that the version doesn't exist
			// A cxx quirk we're using to make returning Option easier
			Err(_) => None,
		}
	}

	/// Return either a Version or None
	///
	/// # Example:
	/// ```
	/// use rust_apt::cache::Cache;
	///
	/// let cache = Cache::new();
	/// let pkg = cache.get("apt").unwrap();
	///
	/// pkg.get_version("2.4.7");
	/// ```
	pub fn get_version(&self, version_str: &str) -> Option<Version<'a>> {
		Some(self.create_version(
			raw::pkg_get_version(&self.ptr.borrow(), version_str.to_string()).ok()?,
		))
	}

	/// Check if the package is installed.
	pub fn is_installed(&self) -> bool { raw::pkg_is_installed(&self.ptr.borrow()) }

	/// Check if the package is upgradable.
	///
	/// ## skip_depcache:
	///
	/// Skipping the DepCache is unnecessary if it's already been initialized.
	/// If you're unsure use `false`
	///
	///   * [true] = Increases performance by skipping the pkgDepCache.
	///   * [false] = Use DepCache to check if the package is upgradable
	pub fn is_upgradable(&self, skip_depcache: bool) -> bool {
		self.depcache
			.borrow()
			.is_upgradable(&self.ptr.borrow(), skip_depcache)
	}

	/// Check if the package is auto installed. (Not installed by the user)
	pub fn is_auto_installed(&self) -> bool {
		self.depcache.borrow().is_auto_installed(&self.ptr.borrow())
	}

	/// Check if the package is auto removable
	pub fn is_auto_removable(&self) -> bool {
		self.depcache.borrow().is_auto_removable(&self.ptr.borrow())
	}

	/// Check if the package is now broken
	pub fn is_now_broken(&self) -> bool { self.depcache.borrow().is_now_broken(&self.ptr.borrow()) }

	/// Check if the package package installed is broken
	pub fn is_inst_broken(&self) -> bool {
		self.depcache.borrow().is_inst_broken(&self.ptr.borrow())
	}

	/// Check if the package is marked install
	pub fn marked_install(&self) -> bool {
		self.depcache.borrow().marked_install(&self.ptr.borrow())
	}

	/// Check if the package is marked upgrade
	pub fn marked_upgrade(&self) -> bool {
		self.depcache.borrow().marked_upgrade(&self.ptr.borrow())
	}

	/// Check if the package is marked purge
	pub fn marked_purge(&self) -> bool { self.depcache.borrow().marked_purge(&self.ptr.borrow()) }

	/// Check if the package is marked delete
	pub fn marked_delete(&self) -> bool { self.depcache.borrow().marked_delete(&self.ptr.borrow()) }

	/// Check if the package is marked keep
	pub fn marked_keep(&self) -> bool { self.depcache.borrow().marked_keep(&self.ptr.borrow()) }

	/// Check if the package is marked downgrade
	pub fn marked_downgrade(&self) -> bool {
		self.depcache.borrow().marked_downgrade(&self.ptr.borrow())
	}

	/// Check if the package is marked reinstall
	pub fn marked_reinstall(&self) -> bool {
		self.depcache.borrow().marked_reinstall(&self.ptr.borrow())
	}

	/// Get the state of a package based on a Mark
	///
	/// ## Returns:
	///   * [true] if the state matches the Mark
	///   * [false] if the state doesn't match the Mark
	///
	/// # Example
	/// ```
	/// use rust_apt::cache::Cache;
	/// use rust_apt::package::Mark;
	///
	/// let cache = Cache::new();
	/// let pkg = cache.get("apt").unwrap();
	///
	/// if pkg.state(&Mark::Auto) {
	///     println!("Package is automatically installed")
	/// } else {
	///     println!("Package is manually installed")
	/// }
	/// ```
	pub fn state(&self, mark: &Mark) -> bool {
		match mark {
			Mark::Keep => self.marked_keep(),
			Mark::Auto => self.is_auto_installed(),
			Mark::Manual => !self.is_auto_installed(),
			Mark::Remove => self.marked_delete(),
			Mark::Purge => self.marked_purge(),
			Mark::Install => self.marked_install(),
			Mark::Reinstall => self.marked_reinstall(),
			Mark::NoReinstall => !self.marked_reinstall(),
			Mark::Upgrade => self.marked_upgrade(),
			Mark::Downgrade => self.marked_downgrade(),
		}
	}

	/// Set a Mark on a package using the Mark enum
	///
	/// ## Returns:
	///   * [true] if the mark was successful
	///   * [false] if the mark was unsuccessful
	///
	/// ## Note:
	/// There are some cases where a mark is successful, but nothing will
	/// change.
	///
	/// If a package is already installed, at the latest version,
	/// and you mark that package for install you will get true,
	/// but the package will not be altered. `get(Mark::Install)` will be false
	///
	/// # Example
	/// ```
	/// use rust_apt::cache::Cache;
	/// use rust_apt::package::Mark;
	///
	/// let cache = Cache::new();
	/// let pkg = cache.get("apt").unwrap();
	///
	/// if pkg.set(&Mark::Purge) {
	///     println!("Package will be purged")
	/// } else {
	///     println!("Package was unable to be purged")
	/// }
	/// ```
	pub fn set(&self, mark: &Mark) -> bool {
		match mark {
			Mark::Keep => self.mark_keep(),
			Mark::Auto => self.mark_auto(true),
			Mark::Manual => self.mark_auto(false),
			Mark::Remove => self.mark_delete(false),
			Mark::Purge => self.mark_delete(true),
			// By default will install dependencies
			Mark::Install => self.mark_install(true, true),
			Mark::Reinstall => self.mark_reinstall(true),
			Mark::NoReinstall => self.mark_reinstall(false),
			Mark::Upgrade => self.mark_install(true, true),
			// Maybe we could make a function that will downgrade
			// The package to the previous version if it still exists
			Mark::Downgrade => unimplemented!(),
		}
	}

	/// # Mark a package as automatically installed.
	///
	/// ## mark_auto:
	///   * [true] = Mark the package as automatically installed.
	///   * [false] = Mark the package as manually installed.
	pub fn mark_auto(&self, mark_auto: bool) -> bool {
		self.depcache
			.borrow()
			.mark_auto(&self.ptr.borrow(), mark_auto);
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
	pub fn mark_keep(&self) -> bool { self.depcache.borrow().mark_keep(&self.ptr.borrow()) }

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
		self.depcache
			.borrow()
			.mark_delete(&self.ptr.borrow(), purge)
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
		self.depcache
			.borrow()
			.mark_install(&self.ptr.borrow(), auto_inst, from_user)
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
		self.depcache
			.borrow()
			.mark_reinstall(&self.ptr.borrow(), reinstall);
		// Convert to a bool to remain consistent with other mark functions/
		true
	}

	/// Protect a package's state
	/// for when [`crate::cache::Cache::resolve`] is called.
	pub fn protect(&self) { self.resolver.borrow().protect(&self.ptr.borrow()) }

	/// Returns a version list
	/// starting with the newest and ending with the oldest.
	pub fn versions(&self) -> impl Iterator<Item = Version<'a>> + '_ {
		pkg_version_list(&self.ptr.borrow()).into_iter().map(|ver| {
			Version::new(
				Rc::clone(&self.records),
				Rc::clone(&self.cache_ptr),
				Rc::clone(&self.depcache),
				Rc::clone(&self.resolver),
				Rc::clone(&self.pointer_map),
				Rc::clone(&self.ptr),
				self.pointer_map.borrow_mut().get_version(ver),
			)
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
	records: Rc<RefCell<Records>>,
	cache_ptr: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
	depcache: Rc<RefCell<DepCache>>,
	resolver: Rc<RefCell<ProblemResolver>>,
	pointer_map: Rc<RefCell<PointerMap>>,
	parent: Rc<RefCell<raw::PackagePtr>>,
	ptr: Rc<RefCell<raw::VersionPtr>>,
	depends_list: OnceCell<HashMap<String, Vec<Dependency>>>,
}

impl<'a> Version<'a> {
	fn new(
		records: Rc<RefCell<Records>>,
		cache_ptr: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
		depcache: Rc<RefCell<DepCache>>,
		resolver: Rc<RefCell<ProblemResolver>>,
		pointer_map: Rc<RefCell<PointerMap>>,
		parent: Rc<RefCell<raw::PackagePtr>>,
		ptr: Rc<RefCell<raw::VersionPtr>>,
	) -> Self {
		Self {
			_lifetime: &PhantomData,
			records,
			cache_ptr,
			depcache,
			resolver,
			pointer_map,
			parent,
			ptr,
			depends_list: OnceCell::new(),
		}
	}

	/// Return the version's parent package.
	pub fn parent(&self) -> Package {
		Package::new(
			Rc::clone(&self.records),
			Rc::clone(&self.cache_ptr),
			Rc::clone(&self.depcache),
			Rc::clone(&self.resolver),
			Rc::clone(&self.pointer_map),
			Rc::clone(&self.parent),
		)
	}

	/// The architecture of the version.
	pub fn arch(&self) -> String { raw::ver_arch(&self.ptr.borrow()) }

	/// The version string of the version. "1.4.10"
	pub fn version(&self) -> String { raw::ver_str(&self.ptr.borrow()) }

	/// The section of the version as shown in `apt show`.
	pub fn section(&self) -> Option<String> { raw::ver_section(&self.ptr.borrow()).ok() }

	/// The list of packages that this package provides for.
	///
	/// The first item in the returned tuple is the package name of the provided
	/// package. The second is the version, if it has been specified.
	/// I.e. if `Provides: rustc` was in a control file, you'd get `("rustc",
	/// None)`. If you had `Provides: rustc (= 1.5)` on the other hand, you'd
	/// get `("rustc", Some("1.5"))`.
	///
	/// If you need to see what packages provide this version's parent package,
	/// use [`Package::rev_provides_list`] instead.
	// TODO: This appears to sometimes return janky result, such as
	// `apt-transport-https` being reporting as providing `apt-transport-https`.
	// Additionally with the new records accessors, is this still necessary?
	pub fn provides_list(&self) -> Vec<(String, Option<String>)> {
		let mut returned_pkgs = Vec::new();

		for pkg in raw::ver_provides_list(&self.ptr.borrow()) {
			let (pkgname, ver) = pkg.split_once('/').unwrap();

			if !ver.is_empty() {
				returned_pkgs.push((pkgname.to_string(), Some(ver.to_string())));
			} else {
				returned_pkgs.push((pkgname.to_string(), None));
			}
		}

		returned_pkgs
	}

	/// The priority string as shown in `apt show`.
	pub fn priority_str(&self) -> Option<String> { raw::ver_priority_str(&self.ptr.borrow()).ok() }

	/// The priority of the package as shown in `apt policy`.
	pub fn priority(&self) -> i32 {
		raw::ver_priority(&self.records.borrow().cache.borrow(), &self.ptr.borrow())
	}

	/// The size of the .deb file.
	pub fn size(&self) -> u64 { raw::ver_size(&self.ptr.borrow()) }

	/// The uncompressed size of the .deb file.
	pub fn installed_size(&self) -> u64 { raw::ver_installed_size(&self.ptr.borrow()) }

	/// The ID of the version.
	pub fn id(&self) -> u32 { raw::ver_id(&self.ptr.borrow()) }

	/// If the version is able to be downloaded.
	pub fn downloadable(&self) -> bool { raw::ver_downloadable(&self.ptr.borrow()) }

	/// Check if the version is installed
	pub fn is_installed(&self) -> bool { raw::ver_installed(&self.ptr.borrow()) }

	/// Set this version as the candidate.
	pub fn set_candidate(&self) { self.depcache.borrow().set_candidate(&self.ptr.borrow()); }

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
	pub fn description(&self) -> Option<String> {
		let mut records = self.records.borrow_mut();
		records.lookup_desc(&self.ptr.borrow().desc);
		records.description()
	}

	/// Get the translated short description
	pub fn summary(&self) -> Option<String> {
		let mut records = self.records.borrow_mut();
		records.lookup_desc(&self.ptr.borrow().desc);
		records.summary()
	}

	/// Get data from the specified record field
	///
	/// # Returns:
	///   * Some String or None if the field doesn't exist.
	///
	/// # Example:
	/// ```
	/// use rust_apt::cache::Cache;
	/// use rust_apt::records::RecordField;
	///
	/// let cache = Cache::new();
	/// let pkg = cache.get("apt").unwrap();
	/// let cand = pkg.candidate().unwrap();
	///
	/// println!("{}", cand.get_record(RecordField::Maintainer).unwrap());
	/// // Or alternatively you can just pass any string
	/// println!("{}", cand.get_record("Description-md5").unwrap());
	/// ```
	pub fn get_record(&self, field_name: &str) -> Option<String> {
		// If the lookup fails it could return data from an unrelated package
		match self.lookup_ver() {
			true => self.records.borrow().get_field(field_name.to_string()),
			false => None,
		}
	}

	/// Internal function to help with lookups and deduplicate code.
	///
	/// It is expected that this may not ever return false except maybe virtual
	/// packages.
	fn lookup_ver(&self) -> bool {
		// It is possible we should do something similar with lookup_desc.
		// May need to research if we should OnceCell the ver_file_list again,
		// the performance penalty of getting the ver_file_list may not be an issue.
		let mut records = self.records.borrow_mut();
		if let Some(ver_file) = ver_file_list(&self.ptr.borrow()).first() {
			records.lookup_ver(ver_file);
			return true;
		}
		false
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
		match self.lookup_ver() {
			true => self.records.borrow().hash_find(hash_type),
			false => None,
		}
	}

	/// Returns an iterator of PackageFiles (Origins) for the version
	pub fn package_files(&self) -> impl Iterator<Item = PackageFile> + '_ {
		ver_pkg_file_list(&self.ptr.borrow())
			.into_iter()
			.map(|pkg_file| PackageFile::new(pkg_file, Rc::clone(&self.records.borrow().cache)))
	}

	/// Returns an iterator of URIs for the version
	pub fn uris(&self) -> impl Iterator<Item = String> + '_ {
		ver_file_list(&self.ptr.borrow())
			.into_iter()
			.filter_map(|ver_file| {
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
				cache_ptr: Rc::clone(&self.cache_ptr),
				depcache: Rc::clone(&self.depcache),
				resolver: Rc::clone(&self.resolver),
				pointer_map: Rc::clone(&self.pointer_map),
				package: Rc::clone(&self.parent),
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
		for dep in raw::dep_list(&self.ptr.borrow()) {
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
			self.section().unwrap_or_else(|| String::from("None")),
			self.priority_str().unwrap_or_else(|| String::from("None")),
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
	cache_ptr: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
	depcache: Rc<RefCell<DepCache>>,
	resolver: Rc<RefCell<ProblemResolver>>,
	pointer_map: Rc<RefCell<PointerMap>>,
	package: Rc<RefCell<raw::PackagePtr>>,
}

impl BaseDep {
	pub fn name(&self) -> &String { &self.apt_dep.name }

	pub fn version(&self) -> &String { &self.apt_dep.version }

	pub fn comp(&self) -> &String { &self.apt_dep.comp }

	pub fn dep_type(&self) -> &String { &self.apt_dep.dep_type }

	pub fn all_targets(&self) -> impl Iterator<Item = Version> {
		raw::dep_all_targets(&self.apt_dep).into_iter().map(|ptr| {
			Version::new(
				Rc::clone(&self.records),
				Rc::clone(&self.cache_ptr),
				Rc::clone(&self.depcache),
				Rc::clone(&self.resolver),
				Rc::clone(&self.pointer_map),
				Rc::clone(&self.package),
				self.pointer_map.borrow_mut().get_version(ptr),
			)
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
		pub fn pkg_current_version(iterator: &PackagePtr) -> Result<VersionPtr>;

		/// Return the candidate version of the package.
		/// Ptr will be NULL if there isn't a candidate.
		pub fn pkg_candidate_version(
			cache: &UniquePtr<PkgCacheFile>,
			iterator: &PackagePtr,
		) -> Result<VersionPtr>;

		/// Return the version determined by a version string.
		pub fn pkg_get_version(iterator: &PackagePtr, version_str: String) -> Result<VersionPtr>;

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

		/// The list of package versions that this package provides for.
		/// Packages are returned as a vector of strings, with each string being
		/// the pkgname and version separated by a '/'. If a provided package
		/// doesn't specify a version (i.e. `Provides: rustc`), the string will
		/// end with a slash (`rustc/`).
		pub fn ver_provides_list(version: &VersionPtr) -> Vec<String>;

		/// The version string of the version. "1.4.10"
		pub fn ver_str(version: &VersionPtr) -> String;

		/// The section of the version as shown in `apt show`.
		pub fn ver_section(version: &VersionPtr) -> Result<String>;

		/// The priority string as shown in `apt show`.
		pub fn ver_priority_str(version: &VersionPtr) -> Result<String>;

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
