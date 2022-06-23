//! Contains the raw bindings to libapt-pkg.
use std::fmt;

impl fmt::Debug for apt::VersionPtr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"VersionPtr: {}:{}",
			apt::ver_name(self),
			apt::ver_str(self)
		)?;
		Ok(())
	}
}

impl fmt::Debug for apt::BaseDep {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"BaseDep <Name: {}, Version: {}, Comp: {}, Type: {}>",
			self.name, self.version, self.comp, self.dep_type,
		)?;
		Ok(())
	}
}

impl fmt::Debug for apt::Records {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "package file: {{ To Be Implemented }}")?;
		Ok(())
	}
}

impl fmt::Debug for apt::PkgCacheFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "PkgCacheFile: {{ To Be Implemented }}")?;
		Ok(())
	}
}

impl fmt::Debug for apt::PkgDepCache {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "PkgDepCache: {{ To Be Implemented }}")?;
		Ok(())
	}
}

impl fmt::Debug for apt::PackagePtr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "PackagePtr: {}", apt::get_fullname(self, false))?;
		Ok(())
	}
}

impl fmt::Debug for apt::PackageFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "package file: {{ To Be Implemented }}")?;
		Ok(())
	}
}

impl fmt::Display for apt::SourceFile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Source< Uri: {}, Filename: {}>", self.uri, self.filename)?;
		Ok(())
	}
}

#[cxx::bridge]
pub mod apt {

	/// Struct representing a Source File.
	///
	/// uri = `http://deb.volian.org/volian/dists/scar/InRelease`
	///
	/// filename = `deb.volian.org_volian_dists_scar_InRelease`
	#[derive(Debug)]
	struct SourceFile {
		uri: String,
		filename: String,
	}

	struct BaseDep {
		name: String,
		version: String,
		comp: String,
		dep_type: String,
		ptr: SharedPtr<DepIterator>,
	}

	struct DepContainer {
		dep_type: String,
		dep_list: Vec<BaseDep>,
	}

	struct PackagePtr {
		ptr: UniquePtr<PkgIterator>,
	}

	struct VersionPtr {
		ptr: UniquePtr<VerIterator>,
		desc: UniquePtr<DescIterator>,
	}

	struct PackageFile {
		ver_file: UniquePtr<VerFileIterator>,
		pkg_file: UniquePtr<PkgFileIterator>,
	}

	struct Records {
		records: UniquePtr<PkgRecords>,
	}

	unsafe extern "C++" {

		type PkgCacheFile;
		type PkgCache;
		type PkgSourceList;
		type PkgRecords;
		type PkgDepCache;

		type PkgIterator;
		type PkgFileIterator;
		type VerIterator;
		type VerFileIterator;
		type DepIterator;
		type DescIterator;

		include!("rust-apt/apt-pkg-c/apt-pkg.h");

		// Main Initializers for apt:

		/// init the config system. This must occur before creating the cache.
		pub fn init_config_system();

		/// Create the CacheFile.
		pub fn pkg_cache_create() -> UniquePtr<PkgCacheFile>;

		/// Create the Package Records.
		pub fn pkg_records_create(pcache: &UniquePtr<PkgCacheFile>) -> Records;

		/// Create the depcache.
		pub fn depcache_create(pcache: &UniquePtr<PkgCacheFile>) -> UniquePtr<PkgDepCache>;

		/// Get the package list uris. This is the files that are updated with
		/// `apt update`.
		pub fn source_uris(pcache: &UniquePtr<PkgCacheFile>) -> Vec<SourceFile>;

		// pub fn pkg_cache_compare_versions(
		// 	cache: &UniquePtr<PkgCacheFile>,
		// 	left: *const c_char,
		// 	right: *const c_char,
		// ) -> i32;

		// Package Functions:

		/// Returns a Vector of all the packages in the cache.
		pub fn pkg_list(cache: &UniquePtr<PkgCacheFile>) -> Vec<PackagePtr>;

		/// Return a Vector of all the packages that provide another. steam:i386
		/// provides steam.
		pub fn pkg_provides_list(
			cache: &UniquePtr<PkgCacheFile>,
			iterator: &PackagePtr,
			cand_only: bool,
		) -> Vec<PackagePtr>;

		/// Return the installed version of the package.
		/// Ptr will be NULL if it's not installed.
		pub fn pkg_current_version(iterator: &PackagePtr) -> VersionPtr;

		/// Return the candidate version of the package.
		/// Ptr will be NULL if there isn't a candidate.
		pub fn pkg_candidate_version(
			cache: &UniquePtr<PkgCacheFile>,
			iterator: &PackagePtr,
		) -> VersionPtr;

		/// Return a Vector of all the versions of a package.
		pub fn pkg_version_list(pkg: &PackagePtr) -> Vec<VersionPtr>;

		/// Return a package by name. Ptr will be NULL if the package doesn't
		/// exist.
		pub fn pkg_cache_find_name(cache: &UniquePtr<PkgCacheFile>, name: String) -> PackagePtr;

		/// Return a package by name and architecture.
		/// Ptr will be NULL if the package doesn't exist.
		pub fn pkg_cache_find_name_arch(
			cache: &UniquePtr<PkgCacheFile>,
			name: String,
			arch: String,
		) -> PackagePtr;

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

		/// Get the architecture of a package.
		pub fn pkg_arch(iterator: &PackagePtr) -> String;

		/// Get the ID of a package.
		pub fn pkg_id(iterator: &PackagePtr) -> i32;

		/// Get the current state of a package.
		pub fn pkg_current_state(iterator: &PackagePtr) -> i32;

		/// Get the installed state of a package.
		pub fn pkg_inst_state(iterator: &PackagePtr) -> i32;

		/// Get the selected state of a package.
		pub fn pkg_selected_state(iterator: &PackagePtr) -> i32;

		/// Version Functions:

		/// Return a Vector of all the package files for a version.
		pub fn pkg_file_list(
			pcache: &UniquePtr<PkgCacheFile>,
			ver: &VersionPtr,
		) -> Vec<PackageFile>;

		/// Return a Vector of all the dependencies of a version.
		pub fn dep_list(version: &VersionPtr) -> Vec<DepContainer>;

		/// The name of the versions Parent Package.
		pub fn ver_name(version: &VersionPtr) -> String;

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
		pub fn ver_size(version: &VersionPtr) -> i32;

		/// The uncompressed size of the .deb file.
		pub fn ver_installed_size(version: &VersionPtr) -> i32;

		/// The ID of the version.
		pub fn ver_id(version: &VersionPtr) -> i32;

		/// If the version is able to be downloaded.
		pub fn ver_downloadable(version: &VersionPtr) -> bool;

		/// Check if the version is currently installed.
		pub fn ver_installed(version: &VersionPtr) -> bool;

		/// DepCache Information Accessors:

		/// Is the Package upgradable?
		pub fn pkg_is_upgradable(cache: &UniquePtr<PkgCacheFile>, iterator: &PackagePtr) -> bool;

		/// Is the Package auto installed? Packages marked as auto installed are
		/// usually depenencies.
		pub fn pkg_is_auto_installed(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr)
			-> bool;

		/// Is the Package able to be auto removed?
		pub fn pkg_is_garbage(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marked for install?
		pub fn pkg_marked_install(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marked for upgrade?
		pub fn pkg_marked_upgrade(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marked for removal?
		pub fn pkg_marked_delete(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marked for keep?
		pub fn pkg_marked_keep(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marked for downgrade?
		pub fn pkg_marked_downgrade(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package marked for reinstall?
		pub fn pkg_marked_reinstall(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the installed Package broken?
		pub fn pkg_is_now_broken(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Is the Package to be installed broken?
		pub fn pkg_is_inst_broken(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;

		/// Package Record Management:

		/// Moves the Records into the correct place.
		pub fn ver_file_lookup(records: &mut Records, pkg_file: &PackageFile);

		/// Moves the Records into the correct place.
		pub fn desc_file_lookup(records: &mut Records, desc: &UniquePtr<DescIterator>);

		/// Return the URI for a version as determined by it's package file.
		/// A version could have multiple package files and multiple URIs.
		pub fn ver_uri(
			records: &Records,
			pcache: &UniquePtr<PkgCacheFile>,
			pkg_file: &PackageFile,
		) -> String;

		/// Return the translated long description of a Package.
		pub fn long_desc(records: &Records) -> String;

		/// Return the translated short description of a Package.
		pub fn short_desc(records: &Records) -> String;

		/// Find the hash of a Version. Returns "KeyError" (lul python) if there
		/// is no hash.
		pub fn hash_find(records: &Records, hash_type: String) -> String;

		/// Dependency Functions:

		/// Return a Vector of all versions that can satisfy a dependency.
		pub fn dep_all_targets(dep: &BaseDep) -> Vec<VersionPtr>;

	}
}
