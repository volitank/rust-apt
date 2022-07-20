//! Contains the raw bindings to libapt-pkg.
use std::fmt;

use cxx::ExternType;

use crate::progress::UpdateProgress;

/// Impl for sending UpdateProgress across the barrier.
unsafe impl ExternType for Box<dyn UpdateProgress> {
	type Id = cxx::type_id!("DynUpdateProgress");
	type Kind = cxx::kind::Trivial;
}

// Begin UpdateProgress trait functions

/// Called on c++ to set the pulse interval.
fn pulse_interval(progress: &mut Box<dyn UpdateProgress>) -> usize { (**progress).pulse_interval() }

/// Called when an item is confirmed to be up-to-date.
fn hit(progress: &mut Box<dyn UpdateProgress>, id: u32, description: String) {
	(**progress).hit(id, description)
}

/// Called when an Item has started to download
fn fetch(progress: &mut Box<dyn UpdateProgress>, id: u32, description: String, file_size: u64) {
	(**progress).fetch(id, description, file_size)
}

/// Called when an Item fails to download
fn fail(
	progress: &mut Box<dyn UpdateProgress>,
	id: u32,
	description: String,
	status: u32,
	error_text: String,
) {
	(**progress).fail(id, description, status, error_text)
}

/// Called periodically to provide the overall progress information
fn pulse(
	progress: &mut Box<dyn UpdateProgress>,
	workers: Vec<apt::Worker>,
	percent: f32,
	total_bytes: u64,
	current_bytes: u64,
	current_cps: u64,
) {
	(**progress).pulse(workers, percent, total_bytes, current_bytes, current_cps)
}

/// Called when an item is successfully and completely fetched.
fn done(progress: &mut Box<dyn UpdateProgress>) { (**progress).done() }

/// Called when progress has started
fn start(progress: &mut Box<dyn UpdateProgress>) { (**progress).start() }

/// Called when progress has finished
fn stop(
	progress: &mut Box<dyn UpdateProgress>,
	fetched_bytes: u64,
	elapsed_time: u64,
	current_cps: u64,
	pending_errors: bool,
) {
	(**progress).stop(fetched_bytes, elapsed_time, current_cps, pending_errors)
}

// End UpdateProgress trait functions

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

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod apt {

	/// Struct representing a Source File.
	#[derive(Debug)]
	struct SourceFile {
		/// `http://deb.volian.org/volian/dists/scar/InRelease`
		uri: String,
		/// `deb.volian.org_volian_dists_scar_InRelease`
		filename: String,
	}

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

	/// A wrapper around the Apt pkgIterator.
	struct PackagePtr {
		ptr: UniquePtr<PkgIterator>,
	}

	/// A wrapper around the Apt verIterator.
	struct VersionPtr {
		ptr: UniquePtr<VerIterator>,
		desc: UniquePtr<DescIterator>,
	}

	/// A wrapper around the Apt verFileIterator and pkgFileIterator.
	struct PackageFile {
		ver_file: UniquePtr<VerFileIterator>,
		pkg_file: UniquePtr<PkgFileIterator>,
	}

	/// A wrapper around the Apt pkgRecords class.
	struct Records {
		records: UniquePtr<PkgRecords>,
	}

	/// A simple representation of an Aquire worker.
	///
	/// TODO: Make this better.
	struct Worker {
		is_current: bool,
		status: String,
		id: u64,
		short_desc: String,
		active_subprocess: String,
		current_size: u64,
		total_size: u64,
		complete: bool,
	}

	/// Enum to determine what will be sorted.
	#[derive(Debug)]
	pub enum Sort {
		/// Disable the sort method.
		Disable,
		/// Enable the sort method.
		Enable,
		/// Reverse the sort method.
		Reverse,
	}

	/// Struct for sorting packages.
	#[derive(Debug)]
	pub struct PackageSort {
		pub names: bool,
		pub upgradable: Sort,
		pub virtual_pkgs: Sort,
		pub installed: Sort,
		pub auto_installed: Sort,
		pub auto_removable: Sort,
	}

	extern "Rust" {
		/// Called on c++ to set the pulse interval.
		fn pulse_interval(progress: &mut DynUpdateProgress) -> usize;

		/// Called when an item is confirmed to be up-to-date.
		fn hit(progress: &mut DynUpdateProgress, id: u32, description: String);

		/// Called when an Item has started to download
		fn fetch(progress: &mut DynUpdateProgress, id: u32, description: String, file_size: u64);

		/// Called when an Item fails to download
		fn fail(
			progress: &mut DynUpdateProgress,
			id: u32,
			description: String,
			status: u32,
			error_text: String,
		);

		/// Called periodically to provide the overall progress information
		fn pulse(
			progress: &mut DynUpdateProgress,
			workers: Vec<Worker>,
			percent: f32,
			total_bytes: u64,
			current_bytes: u64,
			current_cps: u64,
		);

		/// Called when an item is successfully and completely fetched.
		fn done(progress: &mut DynUpdateProgress);

		/// Called when progress has started
		fn start(progress: &mut DynUpdateProgress);

		/// Called when progress has finished
		fn stop(
			progress: &mut DynUpdateProgress,
			fetched_bytes: u64,
			elapsed_time: u64,
			current_cps: u64,
			pending_errors: bool,
		);
	}

	unsafe extern "C++" {

		/// Apt C++ Type
		type PkgCacheFile;
		/// Apt C++ Type
		type PkgCache;
		/// Apt C++ Type
		type PkgSourceList;
		/// Apt C++ Type
		type PkgRecords;
		/// Apt C++ Type
		type PkgDepCache;

		/// Apt C++ Type
		type PkgIterator;
		/// Apt C++ Type
		type PkgFileIterator;
		/// Apt C++ Type
		type VerIterator;
		/// Apt C++ Type
		type VerFileIterator;
		/// Apt C++ Type
		type DepIterator;
		/// Apt C++ Type
		type DescIterator;
		/// Apt C++ Type
		type Configuration;

		type DynUpdateProgress = Box<dyn crate::raw::UpdateProgress>;

		include!("rust-apt/apt-pkg-c/apt-pkg.h");
		include!("rust-apt/apt-pkg-c/progress.h");
		include!("rust-apt/apt-pkg-c/configuration.h");

		// Main Initializers for apt:

		/// init the config system. This must occur before creating the cache.
		pub fn init_config();

		pub fn init_system();

		/// Create the CacheFile.
		pub fn pkg_cache_create() -> UniquePtr<PkgCacheFile>;

		/// Update the package lists, handle errors and return a Result.
		pub fn cache_update(
			cache: &UniquePtr<PkgCacheFile>,
			progress: &mut DynUpdateProgress,
		) -> Result<()>;

		/// Create the Package Records.
		pub fn pkg_records_create(cache: &UniquePtr<PkgCacheFile>) -> Records;

		/// Create the depcache.
		pub fn depcache_create(cache: &UniquePtr<PkgCacheFile>) -> UniquePtr<PkgDepCache>;

		/// Get the package list uris. This is the files that are updated with
		/// `apt update`.
		pub fn source_uris(cache: &UniquePtr<PkgCacheFile>) -> Vec<SourceFile>;

		/// Compares two package versions, `ver1` and `ver2`. The returned
		/// integer's value is mapped to one of the following integers:
		/// - Less than 0: `ver1` is less than `ver2`.
		/// - Equal to 0: `ver1` is equal to `ver2`.
		/// - Greater than 0: `ver1` is greater than `ver2`.
		///
		/// Unless you have a specific need for otherwise, you should probably
		/// use [`crate::util::cmp_versions`] instead.
		pub fn cmp_versions(ver1: String, ver2: String) -> i32;

		/// Returns a string dump of configuration options separated by `\n`
		pub fn config_dump() -> String;

		/// Find a key and return it's value as a string.
		pub fn config_find(key: String, default_value: String) -> String;

		/// Find a file and return it's value as a string.
		pub fn config_find_file(key: String, default_value: String) -> String;

		/// Find a directory and return it's value as a string.
		pub fn config_find_dir(key: String, default_value: String) -> String;

		/// Same as find, but for boolean values.
		pub fn config_find_bool(key: String, default_value: bool) -> bool;

		/// Same as find, but for i32 values.
		pub fn config_find_int(key: String, default_value: i32) -> i32;

		/// Return a vector for an Apt configuration list.
		pub fn config_find_vector(key: String) -> Vec<String>;

		/// Set the given key to the specified value.
		pub fn config_set(key: String, value: String);

		/// Simply check if a key exists.
		pub fn config_exists(key: String) -> bool;

		/// Clears all values from a key.
		///
		/// If the value is a list, the entire list is cleared.
		/// If you need to clear 1 value from a list see `config_clear_value`
		pub fn config_clear(key: String);

		/// Clears all configuratations.
		pub fn config_clear_all();

		/// Clear a single value from a list.
		/// Used for removing one item in an apt configuruation list
		pub fn config_clear_value(key: String, value: String);

		// Package Functions:

		/// Returns a Vector of all the packages in the cache.
		pub fn pkg_list(cache: &UniquePtr<PkgCacheFile>, sort: &PackageSort) -> Vec<PackagePtr>;

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

		/// Return a Vector of all the package files for a version.
		pub fn pkg_file_list(cache: &UniquePtr<PkgCacheFile>, ver: &VersionPtr)
			-> Vec<PackageFile>;

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
		pub fn ver_size(version: &VersionPtr) -> u64;

		/// The uncompressed size of the .deb file.
		pub fn ver_installed_size(version: &VersionPtr) -> u64;

		/// The ID of the version.
		pub fn ver_id(version: &VersionPtr) -> u32;

		/// If the version is able to be downloaded.
		pub fn ver_downloadable(version: &VersionPtr) -> bool;

		/// Check if the version is currently installed.
		pub fn ver_installed(version: &VersionPtr) -> bool;

		/// DepCache Information Accessors:

		/// Is the Package upgradable?
		///
		/// `skip_depcache = true` increases performance by skipping the
		/// pkgDepCache Skipping the depcache is very unnecessary if it's
		/// already been initialized If you're not sure, set `skip_depcache =
		/// false`
		pub fn pkg_is_upgradable(
			cache: &UniquePtr<PkgCacheFile>,
			iterator: &PackagePtr,
			skip_decache: bool,
		) -> bool;

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

		/// The number of packages marked for installation.
		pub fn install_count(cache: &UniquePtr<PkgCacheFile>) -> u32;

		/// The number of packages marked for removal.
		pub fn delete_count(cache: &UniquePtr<PkgCacheFile>) -> u32;

		/// The number of packages marked for keep.
		pub fn keep_count(cache: &UniquePtr<PkgCacheFile>) -> u32;

		/// The number of packages with broken dependencies in the cache.
		pub fn broken_count(cache: &UniquePtr<PkgCacheFile>) -> u32;

		/// The size of all packages to be downloaded.
		pub fn download_size(cache: &UniquePtr<PkgCacheFile>) -> u64;

		/// The amount of space required for installing/removing the packages,"
		///
		/// i.e. the Installed-Size of all packages marked for installation"
		/// minus the Installed-Size of all packages for removal."
		pub fn disk_size(cache: &UniquePtr<PkgCacheFile>) -> i64;

		/// Package Record Management:

		/// Moves the Records into the correct place.
		pub fn ver_file_lookup(records: &mut Records, pkg_file: &PackageFile);

		/// Moves the Records into the correct place.
		pub fn desc_file_lookup(records: &mut Records, desc: &UniquePtr<DescIterator>);

		/// Return the URI for a version as determined by it's package file.
		/// A version could have multiple package files and multiple URIs.
		pub fn ver_uri(
			records: &Records,
			cache: &UniquePtr<PkgCacheFile>,
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
