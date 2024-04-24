pub type RawPackage = raw::PkgIterator;
pub type RawVersion = raw::VerIterator;
pub type RawProvider = raw::PrvIterator;
pub type RawDependency = raw::DepIterator;
pub type RawVersionFile = raw::VerFileIterator;
pub type RawDescriptionFile = raw::DescFileIterator;
pub type RawPackageFile = raw::PkgFileIterator;

use std::fmt;
use std::hash::{Hash, Hasher};

use cxx::UniquePtr;
use paste::paste;

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

	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/package.h");

		type PkgIterator;
		type VerIterator;
		type PrvIterator;
		type DepIterator;

		type VerFileIterator;
		type DescFileIterator;
		type PkgFileIterator;

		type IndexFile = crate::raw::cache::raw::IndexFile;

		// Package Declarations

		/// Get the name of the package without the architecture.
		pub fn name(self: &PkgIterator) -> &str;

		/// Get the architecture of a package.
		pub fn arch(self: &PkgIterator) -> &str;

		/// Get the fullname of the package.
		///
		/// Pretty is a bool that will omit the native arch.
		pub fn fullname(self: &PkgIterator, pretty: bool) -> String;

		/// Get the ID of a package.
		pub fn id(self: &PkgIterator) -> u32;

		/// Get the current state of a package.
		pub fn current_state(self: &PkgIterator) -> u8;

		/// Get the installed state of a package.
		pub fn inst_state(self: &PkgIterator) -> u8;

		/// Get the selected state of a package.
		pub fn selected_state(self: &PkgIterator) -> u8;

		/// True if the package is essential.
		pub fn is_essential(self: &PkgIterator) -> bool;

		#[cxx_name = "Index"]
		pub fn index(self: &PkgIterator) -> u64;
		pub fn unique(self: &PkgIterator) -> UniquePtr<PkgIterator>;
		pub fn raw_next(self: Pin<&mut PkgIterator>);
		pub fn end(self: &PkgIterator) -> bool;

		// Version Declarations

		/// The version string of the version. "1.4.10".
		pub fn version(self: &VerIterator) -> &str;

		/// The Arch of the version. "amd64".
		pub fn arch(self: &VerIterator) -> &str;

		/// Return the version's parent PkgIterator.
		pub fn parent_pkg(self: &VerIterator) -> UniquePtr<PkgIterator>;

		/// The ID of the version.
		pub fn id(self: &VerIterator) -> u32;

		/// The section of the version as shown in `apt show`.
		pub fn section(self: &VerIterator) -> Result<&str>;

		/// The priority string as shown in `apt show`.
		pub fn priority_str(self: &VerIterator) -> Result<&str>;

		/// The size of the .deb file.
		pub fn size(self: &VerIterator) -> u64;

		/// The uncompressed size of the .deb file.
		pub fn installed_size(self: &VerIterator) -> u64;

		/// True if the version is able to be downloaded.
		#[cxx_name = "Downloadable"]
		pub fn is_downloadable(self: &VerIterator) -> bool;

		/// True if the version is currently installed
		pub fn is_installed(self: &VerIterator) -> bool;

		/// Always contains the name, even if it is the same as the binary name
		pub fn source_name(self: &VerIterator) -> &str;

		// Always contains the version string,
		// even if it is the same as the binary version.
		pub fn source_version(self: &VerIterator) -> &str;

		pub fn u_description_file(self: &VerIterator) -> Result<UniquePtr<DescFileIterator>>;

		#[cxx_name = "Index"]
		pub fn index(self: &VerIterator) -> u64;
		pub fn unique(self: &VerIterator) -> UniquePtr<VerIterator>;
		pub fn raw_next(self: Pin<&mut VerIterator>);
		pub fn end(self: &VerIterator) -> bool;

		// Provider Declarations

		/// The name of what this provider provides
		pub fn name(self: &PrvIterator) -> &str;

		pub fn version_str(self: &PrvIterator) -> Result<&str>;

		/// The Target Package that can satisfy this provides
		pub fn target_pkg(self: &PrvIterator) -> UniquePtr<PkgIterator>;

		pub fn parent_pkg(self: &DepIterator) -> UniquePtr<PkgIterator>;

		pub fn parent_ver(self: &DepIterator) -> UniquePtr<VerIterator>;

		/// The Target Version that can satisfy this provides
		pub fn target_ver(self: &PrvIterator) -> UniquePtr<VerIterator>;

		#[cxx_name = "Index"]
		pub fn index(self: &PrvIterator) -> u64;
		pub fn unique(self: &PrvIterator) -> UniquePtr<PrvIterator>;
		pub fn raw_next(self: Pin<&mut PrvIterator>);
		pub fn end(self: &PrvIterator) -> bool;

		// Dependency Declarations
		/// String representation of the dependency compare type
		/// "","<=",">=","<",">","=","!="
		pub fn comp_type(self: &DepIterator) -> Result<&str>;

		// Get the dependency type as a u8
		// #[cxx_name = "DepType"]
		pub fn u8_dep_type(self: &DepIterator) -> u8;

		/// Return True if the dep is reverse, false if normal
		#[cxx_name = "Reverse"]
		pub fn is_reverse(self: &DepIterator) -> bool;

		pub fn target_ver(self: &DepIterator) -> Result<&str>;

		pub fn target_pkg(self: &DepIterator) -> UniquePtr<PkgIterator>;

		/// Returns a CxxVector of VerIterators.
		///
		/// These can not be owned and will need to be mapped with unique.
		pub fn all_targets(self: &DepIterator) -> UniquePtr<CxxVector<VerIterator>>;

		/// Return true if this dep is Or'd with the next. The last dep in the
		/// or group will return False.
		pub fn compare_op(self: &DepIterator) -> bool;

		#[cxx_name = "Index"]
		pub fn index(self: &DepIterator) -> u64;
		pub fn unique(self: &DepIterator) -> UniquePtr<DepIterator>;
		pub fn raw_next(self: Pin<&mut DepIterator>);
		pub fn end(self: &DepIterator) -> bool;

		// PackageFile Declarations

		/// The path to the PackageFile
		pub fn filename(self: &PkgFileIterator) -> Result<&str>;

		/// The Archive of the PackageFile. ex: unstable
		pub fn archive(self: &PkgFileIterator) -> Result<&str>;

		/// The Origin of the PackageFile. ex: Debian
		pub fn origin(self: &PkgFileIterator) -> Result<&str>;

		/// The Codename of the PackageFile. ex: main, non-free
		pub fn codename(self: &PkgFileIterator) -> Result<&str>;

		/// The Label of the PackageFile. ex: Debian
		pub fn label(self: &PkgFileIterator) -> Result<&str>;

		/// The Hostname of the PackageFile. ex: deb.debian.org
		pub fn site(self: &PkgFileIterator) -> Result<&str>;

		/// The Component of the PackageFile. ex: sid
		pub fn component(self: &PkgFileIterator) -> Result<&str>;

		/// The Architecture of the PackageFile. ex: amd64
		pub fn arch(self: &PkgFileIterator) -> Result<&str>;

		/// The Index Type of the PackageFile. Known values are:
		///
		/// Debian Package Index, Debian Translation Index, Debian dpkg status
		/// file,
		pub fn index_type(self: &PkgFileIterator) -> Result<&str>;

		/// The Index number of the PackageFile
		#[cxx_name = "Index"]
		pub fn index(self: &PkgFileIterator) -> u64;
		pub fn unique(self: &PkgFileIterator) -> UniquePtr<PkgFileIterator>;
		pub fn raw_next(self: Pin<&mut PkgFileIterator>);
		pub fn end(self: &PkgFileIterator) -> bool;

		/// VersionFile Declarations

		/// Return the package file associated with this version file.
		pub fn pkg_file(self: &VerFileIterator) -> UniquePtr<PkgFileIterator>;

		#[cxx_name = "Index"]
		pub fn index(self: &VerFileIterator) -> u64;
		pub fn unique(self: &VerFileIterator) -> UniquePtr<VerFileIterator>;
		pub fn raw_next(self: Pin<&mut VerFileIterator>);
		pub fn end(self: &VerFileIterator) -> bool;

		/// Return the package file associated with this desc file.
		pub fn pkg_file(self: &DescFileIterator) -> UniquePtr<PkgFileIterator>;

		#[cxx_name = "Index"]
		pub fn index(self: &DescFileIterator) -> u64;
		pub fn unique(self: &DescFileIterator) -> UniquePtr<DescFileIterator>;
		pub fn raw_next(self: Pin<&mut DescFileIterator>);
		pub fn end(self: &DescFileIterator) -> bool;

		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		unsafe fn u_current_version(self: &PkgIterator) -> UniquePtr<VerIterator>;
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		unsafe fn u_version_list(self: &PkgIterator) -> UniquePtr<VerIterator>;
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		unsafe fn u_provides(self: &PkgIterator) -> UniquePtr<PrvIterator>;
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		unsafe fn u_rev_depends(self: &PkgIterator) -> UniquePtr<DepIterator>;
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		unsafe fn u_provides(self: &VerIterator) -> UniquePtr<PrvIterator>;
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		unsafe fn u_depends(self: &VerIterator) -> UniquePtr<DepIterator>;
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		unsafe fn u_version_file(self: &VerIterator) -> UniquePtr<VerFileIterator>;
	}
}

impl raw::PkgIterator {
	/// Get a pointer the the currently installed version
	pub fn current_version(&self) -> Option<UniquePtr<RawVersion>> {
		unsafe { self.u_current_version().make_safe() }
	}

	/// Get a pointer to the beginning of the VerIterator
	pub fn version_list(&self) -> Option<UniquePtr<RawVersion>> {
		unsafe { self.u_version_list().make_safe() }
	}

	/// Get the providers of this package
	pub fn provides_list(&self) -> Option<UniquePtr<RawProvider>> {
		unsafe { self.u_provides().make_safe() }
	}

	pub fn rev_depends_list(&self) -> Option<UniquePtr<RawDependency>> {
		unsafe { self.u_rev_depends().make_safe() }
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

impl raw::VerIterator {
	/// Returns a list of providers if they exist
	pub fn provides_list(&self) -> Option<UniquePtr<RawProvider>> {
		unsafe { self.u_provides().make_safe() }
	}

	/// Get the raw dependencies if they exist
	pub fn depends(&self) -> Option<UniquePtr<RawDependency>> {
		unsafe { self.u_depends().make_safe() }
	}

	// You go through here to get the package files.
	pub fn version_files(&self) -> Option<UniquePtr<RawVersionFile>> {
		unsafe { self.u_version_file().make_safe() }
	}

	// This is for backend records lookups.
	// You can also get package files from here.
	pub fn description_files(&self) -> Option<UniquePtr<RawDescriptionFile>> {
		self.u_description_file().ok()?.make_safe()
	}
}

impl raw::DepIterator {
	/// The Dependency Type. Depends, Recommends, etc.
	pub fn dep_type(&self) -> DepType { DepType::from(self.u8_dep_type()) }

	/// Returns true if the dependency type is critical.
	///
	/// Depends, PreDepends, Conflicts, Obsoletes, Breaks
	/// will return [true].
	///
	/// Suggests, Recommends, Replaces and Enhances
	/// will return [false].
	pub fn is_critical(&self) -> bool {
		match self.dep_type() {
			DepType::Depends => true,
			DepType::PreDepends => true,
			DepType::Suggests => false,
			DepType::Recommends => false,
			DepType::Conflicts => true,
			DepType::Replaces => false,
			DepType::Obsoletes => true,
			DepType::Breaks => true,
			DepType::Enhances => false,
		}
	}
}

/// DepFlags defined in depcache.h
#[allow(non_upper_case_globals, non_snake_case)]
pub mod DepFlags {
	pub const DepNow: u8 = 1;
	pub const DepInstall: u8 = 2;
	pub const DepCVer: u8 = 4;
	pub const DepGnow: u8 = 8;
	pub const DepGInstall: u8 = 16;
	pub const DepGVer: u8 = 32;
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

impl fmt::Display for DepType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.as_ref()) }
}

pub trait IntoRawIter {
	type Item;
	fn raw_iter(self) -> Self::Item;

	fn make_safe(self) -> Option<Self>
	where
		Self: Sized;

	fn to_vec(self) -> Vec<Self>
	where
		Self: Sized;
}

macro_rules! impl_into_raw_iter {
	($($ty:ty),*) => {$(
		paste!(
			pub struct [<Iter $ty>](UniquePtr<$ty>);

			impl Iterator for [<Iter $ty>] {
				type Item = UniquePtr<$ty>;

				fn next(&mut self) -> Option<Self::Item> {
					if self.0.end() {
						None
					} else {
						let ptr = self.0.unique();
						self.0.pin_mut().raw_next();
						Some(ptr)
					}
				}
			}

			impl IntoRawIter for UniquePtr<$ty> {
				type Item = [<Iter $ty>];

				fn raw_iter(self) -> Self::Item { [<Iter $ty>](self) }

				fn make_safe(self) -> Option<Self> { if self.end() { None } else { Some(self) } }

				fn to_vec(self) -> Vec<Self> { self.raw_iter().collect() }
			}
		);
	)*};
}

impl_into_raw_iter!(
	RawPackage,
	RawVersion,
	RawDependency,
	RawProvider,
	RawVersionFile,
	RawDescriptionFile,
	RawPackageFile
);

// TODO: Maybe make some internal macros and export them so
// this can be used in the higher level package.rs
macro_rules! impl_raw_hash {
	($($ty:ty),*) => {$(
		impl Hash for $ty {
			fn hash<H: Hasher>(&self, state: &mut H) { self.id().hash(state); }
		}

		impl PartialEq for $ty {
			fn eq(&self, other: &$ty) -> bool { self.id() == other.id() }
		}

		impl Eq for $ty {}
	)*};
}

impl_raw_hash!(RawPackage, RawVersion);

#[cfg(test)]
mod raw_tests {
	use crate::raw::cache::raw::PkgCacheFile;
	use crate::raw::package::IntoRawIter;

	#[test]
	fn test() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = PkgCacheFile::new(&debs).unwrap();

		let pkg = cache.find_pkg("apt").unwrap();
		dbg!(pkg.name());

		let installed = pkg.current_version().unwrap();

		dbg!(installed.version());

		for pkg in cache.begin().unwrap().raw_iter() {
			println!("ID: {}", pkg.id());
			println!("Name: {}", pkg.name());
			println!("Arch: {}", pkg.arch());
			println!("FullName: {}", pkg.fullname(false));
			println!("current_state: {}", pkg.current_state());
			println!("inst_state: {}", pkg.inst_state());
			println!("selected_state: {}\n", pkg.selected_state());

			match pkg.version_list() {
				Some(versions) => {
					for ver in versions.raw_iter() {
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
		let cache = PkgCacheFile::new(&debs).unwrap();

		// Check Native Arch
		let pkg = cache.find_pkg("www-browser").unwrap();
		println!("{}:{}", pkg.name(), pkg.arch());

		for provider in pkg.provides_list().unwrap().raw_iter() {
			println!("Provider: {}", provider.name());
			println!(
				"  Pkg: {}, Version: {}",
				provider.target_pkg().name(),
				provider.target_ver().version()
			);
		}

		let pkg = cache.find_pkg("apt").unwrap();
		let cand = pkg.current_version().unwrap();

		for provider in cand.provides_list().unwrap().raw_iter() {
			println!("Provider: {}", provider.name());
			println!(
				"  Pkg: {}, Version: {}",
				provider.target_pkg().name(),
				provider.target_ver().version()
			);
		}
	}

	#[test]
	fn temp_test() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = PkgCacheFile::new(&debs).unwrap();

		let pkg = cache.find_pkg("nala").unwrap();
		for ver in pkg.version_list().unwrap().raw_iter() {
			println!("{}", ver.version());
			println!("{}", ver.id());
			println!("{}", ver.index());
		}
	}

	#[test]
	fn raw_depends() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = PkgCacheFile::new(&debs).unwrap();

		let pkg = cache.find_pkg("apt").unwrap();
		let cand = pkg.current_version().unwrap();

		for dep in cand.depends().unwrap().raw_iter() {
			println!(
				"Dep: {}, Comp Op: {}",
				dep.target_pkg().name(),
				dep.compare_op()
			);
			for dep_ver in dep.all_targets().iter() {
				println!("Version: {}", dep_ver.version())
			}
		}
	}

	#[test]
	fn make_segfault() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = PkgCacheFile::new(&debs).unwrap();

		let pkg = cache.find_pkg("apt").unwrap();
		let cand = pkg.current_version().unwrap();
		let desc_file = cand.u_description_file().unwrap();

		dbg!(desc_file.end());

		println!("Desc Files: {}", desc_file.raw_iter().count());

		// Commented code should not compile

		// dbg!(desc_file.is_null());
		// dbg!(desc_file.end());

		// let pkg_file = desc_file.pkg_file();
		// pkg_file.arch().unwrap();
	}

	#[test]
	fn raw_files() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = PkgCacheFile::new(&debs).unwrap();

		let pkg = cache.find_pkg("apt").unwrap();
		let cand = pkg.current_version().unwrap();
		let depcache = cache.create_depcache();

		// Apt should have a candidate as well as current version
		assert!(depcache.candidate_version(&pkg).is_some());

		let ver_files = cand.version_files().unwrap().to_vec();

		println!("Ver Files: {}", ver_files.len());
		println!(
			"Desc Files: {}",
			cand.u_description_file().unwrap().raw_iter().count()
		);

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
		let cache = PkgCacheFile::new(&debs).unwrap();

		let pkg = cache.find_pkg("apt").unwrap();

		let depcache = cache.create_depcache();
		dbg!(depcache.is_upgradable(&pkg));

		depcache.mark_delete(&pkg, false);

		dbg!(depcache.marked_delete(&pkg));

		dbg!(depcache.delete_count());

		let mut progress = crate::raw::progress::NoOpProgress::new_box();
		depcache.full_upgrade(&mut progress).unwrap();

		for pkg in cache.begin().unwrap().raw_iter() {
			if depcache.marked_upgrade(&pkg) {
				println!("Upgrade => {}", pkg.fullname(false));
			}
		}
	}

	#[test]
	fn source_uris() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = PkgCacheFile::new(&debs).unwrap();
		dbg!(cache.source_uris());
	}

	#[test]
	fn priority() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = PkgCacheFile::new(&debs).unwrap();
		let pkg = cache.find_pkg("apt").unwrap();
		let depcache = cache.create_depcache();
		let cand = unsafe { depcache.u_candidate_version(&pkg) };

		dbg!(cache.priority(&cand));
	}

	#[test]
	fn records() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = PkgCacheFile::new(&debs).unwrap();
		let pkg = cache.find_pkg("apt").unwrap();
		let depcache = cache.create_depcache();
		let cand = unsafe { depcache.u_candidate_version(&pkg) };

		let records = cache.create_records();

		records.ver_file_lookup(&cand.version_files().unwrap().raw_iter().next().unwrap());
		dbg!(records.short_desc().unwrap());
		records.desc_file_lookup(&cand.description_files().unwrap());
		dbg!(records.long_desc().unwrap());

		let pkg_file = cand
			.version_files()
			.unwrap()
			.raw_iter()
			.next()
			.unwrap()
			.pkg_file();
		let index = cache.find_index(&pkg_file);

		dbg!(cache.is_trusted(&index));

		dbg!(records.ver_uri(&index).unwrap());
	}

	#[test]
	fn update() {
		// This test Requires root
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = PkgCacheFile::new(&debs).unwrap();
		let mut progress = crate::raw::progress::AptAcquireProgress::new_box();

		cache.update(&mut progress).unwrap();
	}

	#[test]
	fn pacman() {
		crate::config::init_config_system();

		let debs: Vec<String> = vec![];
		let cache = PkgCacheFile::new(&debs).unwrap();
		let depcache = cache.create_depcache();
		let _pacman = crate::raw::pkgmanager::raw::create_pkgmanager(&depcache);
		let _resolve = crate::raw::pkgmanager::raw::create_problem_resolver(&depcache);
	}
}
