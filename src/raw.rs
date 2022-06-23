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

	// struct Cache {
	// 	cache_file: UniquePtr<PkgCacheFile>,
	// //	cache: UniquePtr<PkgCache>,
	// //	source: UniquePtr<PkgSourceList>,
	// }

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

		type PkgIterator;
		type PkgFileIterator;
		type VerIterator;
		type VerFileIterator;
		type DepIterator;
		type PkgRecords;
		type DescIterator;
		type PkgDepCache;
		include!("rust-apt/apt-pkg-c/apt-pkg.h");

		/// Main Initializers for APT
		pub fn init_config_system();

		pub fn pkg_cache_create() -> UniquePtr<PkgCacheFile>;
		pub unsafe fn pkg_records_create(pcache: &UniquePtr<PkgCacheFile>) -> Records;
		pub unsafe fn depcache_create(pcache: &UniquePtr<PkgCacheFile>) -> UniquePtr<PkgDepCache>;

		pub unsafe fn source_uris(pcache: &UniquePtr<PkgCacheFile>) -> Vec<SourceFile>;
		// pub unsafe fn pkg_cache_compare_versions(
		// 	cache: &UniquePtr<PkgCacheFile>,
		// 	left: *const c_char,
		// 	right: *const c_char,
		// ) -> i32;

		/// Iterator Creators
		pub unsafe fn pkg_list(cache: &UniquePtr<PkgCacheFile>) -> Vec<PackagePtr>;
		pub unsafe fn pkg_file_list(pcache: &UniquePtr<PkgCacheFile>, ver: &VersionPtr) -> Vec<PackageFile>;
		pub unsafe fn pkg_provides_list(
			cache: &UniquePtr<PkgCacheFile>,
			iterator: &PackagePtr,
			cand_only: bool,
		) -> Vec<PackagePtr>;

		pub unsafe fn pkg_current_version(iterator: &PackagePtr) -> VersionPtr;
		pub unsafe fn pkg_candidate_version(
			cache: &UniquePtr<PkgCacheFile>,
			iterator: &PackagePtr,
		) -> VersionPtr;
		pub unsafe fn pkg_version_list(pkg: &PackagePtr) -> Vec<VersionPtr>;

		pub unsafe fn pkg_cache_find_name(cache: &UniquePtr<PkgCacheFile>, name: String) -> PackagePtr;
		pub unsafe fn pkg_cache_find_name_arch(
			cache: &UniquePtr<PkgCacheFile>,
			name: String,
			arch: String,
		) -> PackagePtr;

		/// Information Accessors
		pub unsafe fn pkg_is_upgradable(cache: &UniquePtr<PkgCacheFile>, iterator: &PackagePtr) -> bool;
		pub unsafe fn pkg_is_auto_installed(
			cache: &UniquePtr<PkgCacheFile>,
			wrapper: &PackagePtr,
		) -> bool;
		pub unsafe fn pkg_is_garbage(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;
		pub unsafe fn pkg_marked_install(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;
		pub unsafe fn pkg_marked_upgrade(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;
		pub unsafe fn pkg_marked_delete(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;
		pub unsafe fn pkg_marked_keep(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;
		pub unsafe fn pkg_marked_downgrade(
			cache: &UniquePtr<PkgCacheFile>,
			wrapper: &PackagePtr,
		) -> bool;
		pub unsafe fn pkg_marked_reinstall(
			cache: &UniquePtr<PkgCacheFile>,
			wrapper: &PackagePtr,
		) -> bool;
		pub unsafe fn pkg_is_now_broken(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;
		pub unsafe fn pkg_is_inst_broken(cache: &UniquePtr<PkgCacheFile>, wrapper: &PackagePtr) -> bool;
		pub unsafe fn pkg_is_installed(iterator: &PackagePtr) -> bool;
		pub unsafe fn pkg_has_versions(iterator: &PackagePtr) -> bool;
		pub unsafe fn pkg_has_provides(iterator: &PackagePtr) -> bool;
		pub fn get_fullname(iterator: &PackagePtr, pretty: bool) -> String;
		// pub unsafe fn pkg_name(iterator: &PackagePtr) -> String;
		pub unsafe fn pkg_arch(iterator: &PackagePtr) -> String;
		pub unsafe fn Arch(self: &PackagePtr) -> String;
		pub unsafe fn pkg_id(iterator: &PackagePtr) -> i32;
		pub unsafe fn pkg_current_state(iterator: &PackagePtr) -> i32;
		pub unsafe fn pkg_inst_state(iterator: &PackagePtr) -> i32;
		pub unsafe fn pkg_selected_state(iterator: &PackagePtr) -> i32;
		pub unsafe fn pkg_essential(iterator: &PackagePtr) -> bool;

		// pub unsafe fn dep_list(version: &VersionPtr) -> Vec<DepContainer>;
		pub unsafe fn dep_list(version: &VersionPtr) -> Vec<DepContainer>;
		pub unsafe fn ver_arch(version: &VersionPtr) -> String;
		pub fn ver_str(version: &VersionPtr) -> String;
		pub unsafe fn ver_section(version: &VersionPtr) -> String;
		pub unsafe fn ver_priority_str(version: &VersionPtr) -> String;
		pub unsafe fn ver_priority(cache: &UniquePtr<PkgCacheFile>, version: &VersionPtr) -> i32;
		// pub unsafe fn ver_source_package(version: VersionPtr) -> *const
		// c_char; pub unsafe fn ver_source_version(version: VersionPtr) ->
		// *const c_char;
		pub fn ver_name(version: &VersionPtr) -> String;
		pub unsafe fn ver_size(version: &VersionPtr) -> i32;
		pub unsafe fn ver_installed_size(version: &VersionPtr) -> i32;
		pub unsafe fn ver_downloadable(version: &VersionPtr) -> bool;
		pub unsafe fn ver_id(version: &VersionPtr) -> i32;
		pub unsafe fn ver_installed(version: &VersionPtr) -> bool;

		/// Package Records Management
		pub unsafe fn ver_file_lookup(records: &mut Records, pkg_file: &PackageFile);
		pub unsafe fn desc_file_lookup(records: &mut Records, desc: &UniquePtr<DescIterator>);
		pub unsafe fn ver_uri(
			records: &Records,
			pcache: &UniquePtr<PkgCacheFile>,
			pkg_file: &PackageFile,
		) -> String;
		pub unsafe fn long_desc(records: &Records) -> String;
		pub unsafe fn short_desc(records: &Records) -> String;
		pub unsafe fn hash_find(records: &Records, hash_type: String) -> String;

		pub unsafe fn dep_all_targets(dep: &BaseDep) -> Vec<VersionPtr>;
		// pub unsafe fn long_desc(
		// 	cache: &UniquePtr<PkgCacheFile>,
		// 	records: *mut PkgRecords,
		// 	iterator: &PackagePtr,
		// ) -> String;

		// Unused Functions
		// They may be used in the future
		// pub unsafe fn validate(version: VersionPtr, depcache: &UniquePtr<PkgCacheFile>) ->
		// bool; pub unsafe fn ver_iter_dep_iter(version: VersionPtr) -> *mut
		// DepIterator; pub unsafe fn dep_iter_release(iterator: *mut DepIterator);

		// pub unsafe fn dep_iter_next(iterator: *mut DepIterator);
		// pub unsafe fn dep_iter_end(iterator: *mut DepIterator) -> bool;

		// pub fn dep_iter_target_pkg(iterator: *mut DepIterator) -> &PackagePtr;
		// pub fn dep_iter_target_ver(iterator: *mut DepIterator) -> *const c_char;
		// pub fn dep_iter_comp_type(iterator: *mut DepIterator) -> *const c_char;
		// pub fn dep_iter_dep_type(iterator: *mut DepIterator) -> *const c_char;

		// pub fn ver_file_parser_short_desc(parser: VerFileParser) -> *mut c_char;
		// pub fn ver_file_parser_long_desc(parser: VerFileParser) -> *mut c_char;

		// pub fn ver_file_parser_maintainer(parser: VerFileParser) -> *mut c_char;
		// pub fn ver_file_parser_homepage(parser: VerFileParser) -> *mut c_char;

		// pub unsafe fn pkg_file_iter_next(iterator: *mut PkgFileIterator);
		// pub unsafe fn pkg_file_iter_end(iterator: *mut PkgFileIterator) -> bool;

		// pub unsafe fn pkg_file_iter_file_name(iterator: *mut PkgFileIterator) ->
		// *const c_char; pub unsafe fn pkg_file_iter_archive(iterator: *mut
		// PkgFileIterator) -> *const c_char; pub unsafe fn
		// pkg_file_iter_version(iterator: *mut PkgFileIterator) -> *const c_char;
		// pub unsafe fn pkg_file_iter_origin(iterator: *mut PkgFileIterator) -> *const
		// c_char; pub unsafe fn pkg_file_iter_codename(iterator: *mut PkgFileIterator)
		// -> *const c_char; pub unsafe fn pkg_file_iter_label(iterator: *mut
		// PkgFileIterator) -> *const c_char; pub unsafe fn pkg_file_iter_site(iterator:
		// *mut PkgFileIterator) -> *const c_char; pub unsafe fn
		// pkg_file_iter_component(iterator: *mut PkgFileIterator) -> *const c_char; pub
		// unsafe fn pkg_file_iter_architecture(iterator: *mut PkgFileIterator) ->
		// *const c_char; pub unsafe fn pkg_file_iter_index_type(iterator: *mut
		// PkgFileIterator) -> *const c_char;
	}
}
