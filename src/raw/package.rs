pub type RawPackage = raw::Package;
pub type RawVersion = raw::Version;
pub type RawProvider = raw::Provider;
pub type RawDependency = raw::Dependency;
pub type RawVersionFile = raw::VersionFile;
pub type RawDescriptionFile = raw::DescriptionFile;
pub type RawPackageFile = raw::PackageFile;
use std::hash::{Hash, Hasher};

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {
	// Some weirdness exists in import order.
	// SourceURI is defined here, but used in the cache.
	// We need to impl vec so it can be put in one.
	impl Vec<SourceURI> {}
	#[derive(Debug)]
	pub struct SourceURI {
		pub uri: String,
		pub path: String,
	}

	pub struct Package {
		ptr: UniquePtr<PkgIterator>,
	}

	pub struct Version {
		ptr: UniquePtr<VerIterator>,
	}

	pub struct Provider {
		ptr: UniquePtr<PrvIterator>,
	}

	pub struct Dependency {
		ptr: UniquePtr<DepIterator>,
	}

	pub struct VersionFile {
		ptr: UniquePtr<VerFileIterator>,
	}

	pub struct DescriptionFile {
		ptr: UniquePtr<DescFileIterator>,
	}

	pub struct PackageFile {
		ptr: UniquePtr<PkgFileIterator>,
		index_file: UniquePtr<IndexFile>,
	}

	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/types.h");
		include!("rust-apt/apt-pkg-c/package.h");

		type PkgIterator;
		type VerIterator;
		type PrvIterator;
		type DepIterator;

		type VerFileIterator;
		type DescFileIterator;
		type PkgFileIterator;
		type IndexFile;

		// Package Declarations

		/// Get the name of the package without the architecture.
		pub fn name(self: &Package) -> &str;

		/// Get the architecture of a package.
		pub fn arch(self: &Package) -> &str;

		/// Get the fullname of the package.
		///
		/// Pretty is a bool that will omit the native arch.
		pub fn fullname(self: &Package, pretty: bool) -> String;

		/// Get the ID of a package.
		pub fn id(self: &Package) -> u32;

		/// Get the current state of a package.
		pub fn current_state(self: &Package) -> u8;

		/// Get the installed state of a package.
		pub fn inst_state(self: &Package) -> u8;

		/// Get the selected state of a package.
		pub fn selected_state(self: &Package) -> u8;

		/// Get a pointer the the currently installed version
		///
		/// Safety: If Version.end() is true,
		/// calling methods on the Version can segfault.
		pub fn unsafe_current_version(self: &Package) -> Version;

		/// Get a pointer to the beginning of the VerIterator
		///
		/// Safety: If Version.end() is true,
		/// calling methods on the Version can segfault.
		pub fn unsafe_version_list(self: &Package) -> Version;

		/// Get the providers of this package
		pub fn unsafe_provides(self: &Package) -> Provider;

		pub fn unsafe_rev_depends(self: &Package) -> Dependency;

		/// True if the package is essential.
		pub fn is_essential(self: &Package) -> bool;

		pub fn raw_next(self: &Package);

		/// This will tell you if the inner PkgIterator is null
		///
		/// The cxx is_null function will still show non null because of
		/// wrappers in c++
		pub fn end(self: &Package) -> bool;

		// A simple way to clone the pointer
		pub fn unique(self: &Package) -> Package;

		// Version Declarations

		/// The version string of the version. "1.4.10".
		pub fn version(self: &Version) -> &str;

		/// The Arch of the version. "amd64".
		pub fn arch(self: &Version) -> &str;

		/// The ID of the version.
		pub fn id(self: &Version) -> u32;

		/// The section of the version as shown in `apt show`.
		pub fn section(self: &Version) -> Result<&str>;

		/// The priority string as shown in `apt show`.
		pub fn priority_str(self: &Version) -> Result<&str>;

		/// The size of the .deb file.
		pub fn size(self: &Version) -> u64;

		/// The uncompressed size of the .deb file.
		pub fn installed_size(self: &Version) -> u64;

		/// True if the version is able to be downloaded.
		pub fn is_downloadable(self: &Version) -> bool;

		/// True if the version is currently installed
		pub fn is_installed(self: &Version) -> bool;

		/// Always contains the name, even if it is the same as the binary name
		pub fn source_name(self: &Version) -> &str;

		// Always contains the version string,
		// even if it is the same as the binary version.
		pub fn source_version(self: &Version) -> &str;

		pub fn unsafe_provides(self: &Version) -> Provider;

		pub fn unsafe_depends(self: &Version) -> Dependency;

		// This is for backend records lookups.
		// You can also get package files from here.
		pub fn unsafe_description_file(self: &Version) -> DescriptionFile;

		// You go through here to get the package files.
		pub fn unsafe_version_file(self: &Version) -> VersionFile;

		/// Return the parent package. TODO: This probably isn't going to work
		/// rn pub fn parent(self: &Package) -> bool;
		pub fn raw_next(self: &Version);

		/// This will tell you if the inner PkgIterator is null
		///
		/// The cxx is_null function will still show non null because of
		/// wrappers in c++
		pub fn end(self: &Version) -> bool;

		// A simple way to clone the pointer
		pub fn unique(self: &Version) -> Version;

		// Provider Declarations

		/// The name of what this provider provides
		pub fn name(self: &Provider) -> &str;

		pub fn version_str(self: &Provider) -> Result<&str>;

		/// The Target Package that can satisfy this provides
		pub fn target_pkg(self: &Provider) -> Package;

		pub fn parent_pkg(self: &Dependency) -> Package;

		/// The Target Version that can satisfy this provides
		pub fn target_ver(self: &Provider) -> Version;

		pub fn raw_next(self: &Provider);

		// A simple way to clone the pointer
		pub fn unique(self: &Provider) -> Provider;

		/// This will tell you if the inner PkgIterator is null
		///
		/// The cxx is_null function will still show non null because of
		/// wrappers in c++
		pub fn end(self: &Provider) -> bool;

		// Dependency Declarations
		/// String representation of the dependency compare type
		/// "","<=",">=","<",">","=","!="
		pub fn comp_type(self: &Dependency) -> Result<&str>;

		pub fn index(self: &Dependency) -> u32;

		/// Should probably maybe potentially convert this to a method in rust?
		/// Just gets the dependency type. Taken from 'pkgcache.cc
		/// pkgCache::DepType'
		pub fn dep_type(self: &Dependency) -> u8;

		pub fn target_ver(self: &Dependency) -> Result<&str>;

		pub fn target_pkg(self: &Dependency) -> Package;

		pub fn all_targets(self: &Dependency) -> Version;

		/// Return true if this dep is Or'd with the next. The last dep in the
		/// or group will return False.
		pub fn compare_op(self: &Dependency) -> bool;

		/// Increment the Dep Iterator once
		pub fn raw_next(self: &Dependency);
		/// Is the pointer null, basically
		pub fn end(self: &Dependency) -> bool;

		// A simple way to clone the pointer
		pub fn unique(self: &Dependency) -> Dependency;

		// PackageFile Declarations

		/// The path to the PackageFile
		pub fn filename(self: &PackageFile) -> Result<&str>;

		/// The Archive of the PackageFile. ex: unstable
		pub fn archive(self: &PackageFile) -> Result<&str>;

		/// The Origin of the PackageFile. ex: Debian
		pub fn origin(self: &PackageFile) -> Result<&str>;

		/// The Codename of the PackageFile. ex: main, non-free
		pub fn codename(self: &PackageFile) -> Result<&str>;

		/// The Label of the PackageFile. ex: Debian
		pub fn label(self: &PackageFile) -> Result<&str>;

		/// The Hostname of the PackageFile. ex: deb.debian.org
		pub fn site(self: &PackageFile) -> Result<&str>;

		/// The Component of the PackageFile. ex: sid
		pub fn component(self: &PackageFile) -> Result<&str>;

		/// The Architecture of the PackageFile. ex: amd64
		pub fn arch(self: &PackageFile) -> Result<&str>;

		/// The Index Type of the PackageFile. Known values are:
		///
		/// Debian Package Index, Debian Translation Index, Debian dpkg status
		/// file,
		pub fn index_type(self: &PackageFile) -> Result<&str>;

		/// The Index number of the PackageFile
		pub fn index(self: &PackageFile) -> u64;

		// /// VersionFile Declarations

		/// Return the package file associated with this version file.
		pub fn pkg_file(self: &VersionFile) -> PackageFile;

		/// Increment the iterator
		pub fn raw_next(self: &VersionFile);

		pub fn index(self: &VersionFile) -> u64;

		pub fn end(self: &VersionFile) -> bool;

		// A simple way to clone the pointer
		pub fn unique(self: &VersionFile) -> VersionFile;

		/// Return the package file associated with this version file.
		pub fn pkg_file(self: &DescriptionFile) -> PackageFile;

		/// Increment the iterator
		pub fn raw_next(self: &DescriptionFile);

		pub fn index(self: &DescriptionFile) -> u64;

		pub fn end(self: &DescriptionFile) -> bool;

		// A simple way to clone the pointer
		pub fn unique(self: &DescriptionFile) -> DescriptionFile;

	}
}

impl raw::Package {
	pub fn current_version(&self) -> Option<RawVersion> {
		let ver_list = self.unsafe_current_version();

		match ver_list.end() {
			true => None,
			false => Some(ver_list),
		}
	}

	pub fn version_list(&self) -> Option<RawVersion> {
		let ver_list = self.unsafe_version_list();

		match ver_list.end() {
			true => None,
			false => Some(ver_list),
		}
	}

	pub fn provides_list(&self) -> Option<RawProvider> {
		let ver_list = self.unsafe_provides();

		match ver_list.end() {
			true => None,
			false => Some(ver_list),
		}
	}

	pub fn rev_depends_list(&self) -> Option<RawDependency> {
		let rev_dep_list = self.unsafe_rev_depends();

		match rev_dep_list.end() {
			true => None,
			false => Some(rev_dep_list),
		}
	}

	/// True if the Package is installed.
	pub fn is_installed(&self) -> bool { self.current_version().is_some() }

	/// True if the package has versions.
	///
	/// If a package has no versions it is considered virtual.
	pub fn has_versions(&self) -> bool { self.version_list().is_some() }

	/// True if the package provides any other packages.
	pub fn has_provides(&self) -> bool { self.provides_list().is_some() }
}

impl raw::Version {
	pub fn provides_list(&self) -> Option<RawProvider> {
		let ver_list = self.unsafe_provides();

		match ver_list.end() {
			true => None,
			false => Some(ver_list),
		}
	}

	pub fn depends(&self) -> Option<RawDependency> {
		let ver_list = self.unsafe_depends();

		match ver_list.end() {
			true => None,
			false => Some(ver_list),
		}
	}

	pub fn version_files(&self) -> Option<RawVersionFile> {
		let ver_list = self.unsafe_version_file();

		match ver_list.end() {
			true => None,
			false => Some(ver_list),
		}
	}

	pub fn description_files(&self) -> Option<RawDescriptionFile> {
		let ver_list = self.unsafe_description_file();

		match ver_list.end() {
			true => None,
			false => Some(ver_list),
		}
	}
}

macro_rules! raw_iter {
	($structname: ident) => {
		impl Iterator for $structname {
			type Item = $structname;

			fn next(&mut self) -> Option<Self::Item> {
				match self.end() {
					true => None,
					false => {
						let ptr = self.unique();
						self.raw_next();
						Some(ptr)
					},
				}
			}
		}
	};
}

raw_iter!(RawPackage);
raw_iter!(RawVersion);
raw_iter!(RawProvider);
raw_iter!(RawDependency);
raw_iter!(RawVersionFile);
raw_iter!(RawDescriptionFile);

// TODO: Maybe make some internal macros and export them so
// this can be used in the higher level package.rs
macro_rules! raw_hash {
	($structname: ident) => {
		impl Hash for $structname {
			fn hash<H: Hasher>(&self, state: &mut H) { self.id().hash(state); }
		}

		impl PartialEq for $structname {
			fn eq(&self, other: &$structname) -> bool { self.id() == other.id() }
		}

		impl Eq for $structname {}
	};
}

raw_hash!(RawPackage);
raw_hash!(RawVersion);

#[cfg(test)]
mod raw_tests {
	use crate::raw::cache::raw::create_cache;
	use crate::raw::package::RawVersionFile;

	#[test]
	fn test() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = create_cache(&debs).unwrap();

		let pkg = cache.find_pkg("apt").unwrap();
		dbg!(pkg.name());

		let installed = pkg.current_version().unwrap();

		dbg!(installed.version());

		for pkg in cache.begin().unwrap() {
			println!("ID: {}", pkg.id());
			println!("Name: {}", pkg.name());
			println!("Arch: {}", pkg.arch());
			println!("FullName: {}", pkg.fullname(false));
			println!("current_state: {}", pkg.current_state());
			println!("inst_state: {}", pkg.inst_state());
			println!("selected_state: {}\n", pkg.selected_state());

			match pkg.version_list() {
				Some(versions) => {
					for ver in versions {
						println!("Version of '{}'", pkg.name());

						println!("    Version: {}", &ver.version());
						println!("    Arch: {}", &ver.arch());
						println!("    Section: {}", &ver.section().unwrap_or_default());
						println!("    Source Pkg: {}", &ver.source_name());
						println!("    Source Version: {}\n", &ver.source_version());

						println!("End: {}\n\n", pkg.end());
					}
				},
				None => {
					println!("'{}' is a Virtual Package\n", pkg.name());
				},
			}
		}
	}

	#[test]
	fn raw_provides() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = create_cache(&debs).unwrap();

		// Check Native Arch
		let pkg = cache.find_pkg("www-browser").unwrap();
		println!("{}:{}", pkg.name(), pkg.arch());

		for provider in pkg.provides_list().unwrap() {
			println!("Provider: {}", provider.name());
			println!(
				"  Pkg: {}, Version: {}",
				provider.target_pkg().name(),
				provider.target_ver().version()
			);
		}

		let pkg = cache.find_pkg("apt").unwrap();
		let cand = pkg.current_version().unwrap();

		for provider in cand.provides_list().unwrap() {
			println!("Provider: {}", provider.name());
			println!(
				"  Pkg: {}, Version: {}",
				provider.target_pkg().name(),
				provider.target_ver().version()
			);
		}
	}

	#[test]
	fn raw_depends() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = create_cache(&debs).unwrap();

		let pkg = cache.find_pkg("apt").unwrap();
		let cand = pkg.current_version().unwrap();

		for dep in cand.depends().unwrap() {
			println!(
				"Dep: {}, Comp Op: {}",
				dep.target_pkg().name(),
				dep.compare_op()
			);
			for dep_ver in dep.all_targets() {
				println!("Version: {}", dep_ver.version())
			}
		}
	}

	#[test]
	fn raw_files() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = create_cache(&debs).unwrap();

		let pkg = cache.find_pkg("apt").unwrap();
		let cand = pkg.current_version().unwrap();
		let depcache = cache.create_depcache();

		// Apt should have a candidate as well as current version
		assert!(!depcache.unsafe_candidate_version(&pkg).ptr.is_null());

		let ver_files: Vec<RawVersionFile> = cand.version_files().unwrap().collect();

		println!("Ver Files: {}", ver_files.len());
		println!("Desc Files: {}", cand.description_files().unwrap().count());

		for file in ver_files {
			let pkg_file = file.pkg_file();
			#[rustfmt::skip] // Skip Formatting the string.
			println!(
				"PackageFile: {{\n  \
				  FileName: {},\n  \
				  Archive: {},\n  \
				  Origin: {}\n  \
				  Label: {}\n  \
				  Site: {}\n  \
				  Arch: {}\n  \
				  Component: {}\n  \
				  Index Type: {}\n  \
				  Index: {}\n\
				}}",
				pkg_file.filename().unwrap_or("Unknown"),
				pkg_file.archive().unwrap_or("Unknown"),
				pkg_file.origin().unwrap_or("Unknown"),
				pkg_file.label().unwrap_or("Unknown"),
				pkg_file.site().unwrap_or("Unknown"),
				pkg_file.arch().unwrap_or("Unknown"),
				pkg_file.component().unwrap_or("Unknown"),
				pkg_file.index_type().unwrap_or("Unknown"),
				pkg_file.index(),
			);
		}
	}

	#[test]
	fn raw_depcache() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = create_cache(&debs).unwrap();

		let pkg = cache.find_pkg("apt").unwrap();

		let depcache = cache.create_depcache();
		dbg!(depcache.is_upgradable(&pkg));

		depcache.mark_delete(&pkg, false);

		dbg!(depcache.marked_delete(&pkg));

		dbg!(depcache.delete_count());

		let mut progress = crate::raw::progress::NoOpProgress::new_box();
		depcache.full_upgrade(&mut progress).unwrap();

		for pkg in cache.begin().unwrap() {
			if depcache.marked_upgrade(&pkg) {
				println!("Upgrade => {}", pkg.fullname(false));
			}
		}
	}

	#[test]
	fn source_uris() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = create_cache(&debs).unwrap();
		dbg!(cache.source_uris());
	}

	#[test]
	fn priority() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = create_cache(&debs).unwrap();
		let pkg = cache.find_pkg("apt").unwrap();
		let depcache = cache.create_depcache();
		let cand = depcache.unsafe_candidate_version(&pkg);

		dbg!(cache.priority(&cand));
	}

	#[test]
	fn records() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = create_cache(&debs).unwrap();
		let pkg = cache.find_pkg("apt").unwrap();
		let depcache = cache.create_depcache();
		let cand = depcache.unsafe_candidate_version(&pkg);

		let records = cache.create_records();

		records.ver_file_lookup(&cand.version_files().unwrap().next().unwrap());
		dbg!(records.short_desc().unwrap());
		records.desc_file_lookup(&cand.description_files().unwrap().next().unwrap());
		dbg!(records.long_desc().unwrap());

		let mut pkg_file = cand.version_files().unwrap().next().unwrap().pkg_file();
		cache.find_index(&mut pkg_file);

		dbg!(cache.is_trusted(&mut pkg_file));

		dbg!(records.ver_uri(&pkg_file).unwrap());
	}

	#[test]
	fn update() {
		// This test Requires root
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = create_cache(&debs).unwrap();
		let mut progress = crate::raw::progress::AptAcquireProgress::new_box();

		cache.update(&mut progress).unwrap();
	}

	#[test]
	fn pacman() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = create_cache(&debs).unwrap();
		let _pacman = crate::raw::pkgmanager::raw::create_pkgmanager(&cache);
		let _resolve = crate::raw::pkgmanager::raw::create_problem_resolver(&cache);
	}
}
