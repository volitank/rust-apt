use cxx::{type_id, ExternType};
/// In general:
///  * `*mut c_void` are to be released by the appropriate function
///  * `*const c_chars` are short-term borrows
///  * `*mut c_chars` are to be freed by `libc::free`.
use std::ffi;
use std::os::raw::c_char;

#[cxx::bridge]
pub mod apt {

	unsafe extern "C++" {
		type PCache;
		type PkgIterator;
		type PkgFileIterator;
		type VerIterator;
		type VerFileIterator;
		type DepIterator;
		type VerFileParser;
		type PkgRecords;
		include!("rust-apt/apt-pkg-c/apt-pkg.h");

		pub fn init_config_system();
		pub fn pkg_cache_create() -> *mut PCache;
		pub unsafe fn depcache_init(pcache: *mut PCache);
		pub unsafe fn pkg_cache_release(cache: *mut PCache);

		pub unsafe fn pkg_cache_compare_versions(
			cache: *mut PCache,
			left: *const c_char,
			right: *const c_char,
		) -> i32;

		// Package iterators
		// =================

		pub unsafe fn pkg_begin(cache: *mut PCache) -> *mut PkgIterator;
		pub unsafe fn pkg_cache_find_name(
			cache: *mut PCache,
			name: *const c_char,
		) -> *mut PkgIterator;
		pub unsafe fn pkg_cache_find_name_arch(
			cache: *mut PCache,
			name: *const c_char,
			arch: *const c_char,
		) -> *mut PkgIterator;
		pub unsafe fn pkg_release(iterator: *mut PkgIterator);

		// pkgCache::PkgIterator
		pub unsafe fn pkg_next(iterator: *mut PkgIterator);
		pub unsafe fn ver_next(iterator: *mut VerIterator);
		pub unsafe fn pkg_end(iterator: *mut PkgIterator) -> bool;
		pub unsafe fn ver_end(iterator: *mut VerIterator) -> bool;

		// Package iterator accessors
		// ==========================
		pub unsafe fn pkg_clone(iterator: *mut PkgIterator) -> *mut PkgIterator;
		pub unsafe fn pkg_has_versions(iterator: *mut PkgIterator) -> bool;
		pub unsafe fn pkg_has_provides(iterator: *mut PkgIterator) -> bool;
		pub unsafe fn pkg_is_upgradable(cache: *mut PCache, iterator: *mut PkgIterator) -> bool;
		pub unsafe fn pkg_name(iterator: *mut PkgIterator) -> *const c_char;
		pub unsafe fn get_fullname(iterator: *mut PkgIterator, pretty: bool) -> String;
		pub unsafe fn pkg_arch(iterator: *mut PkgIterator) -> *const c_char;
		pub unsafe fn pkg_current_version(iterator: *mut PkgIterator) -> *mut VerIterator;
		pub unsafe fn pkg_candidate_version(
			cache: *mut PCache,
			iterator: *mut PkgIterator,
		) -> *mut VerIterator;
		pub unsafe fn validate(iterator: *mut VerIterator, depcache: *mut PCache) -> bool;

		// Version iterators
		// =================

		pub unsafe fn pkg_version_list(pkg: *mut PkgIterator) -> *mut VerIterator;
		pub unsafe fn ver_release(iterator: *mut VerIterator);

		// Version accessors
		// =================

		pub unsafe fn ver_str(iterator: *mut VerIterator) -> *const c_char;
		pub unsafe fn ver_section(iterator: *mut VerIterator) -> *const c_char;
		pub unsafe fn ver_source_package(iterator: *mut VerIterator) -> *const c_char;
		pub unsafe fn ver_source_version(iterator: *mut VerIterator) -> *const c_char;
		pub unsafe fn ver_arch(iterator: *mut VerIterator) -> *const c_char;
		pub unsafe fn ver_priority_str(iterator: *mut VerIterator) -> *const c_char;
		pub unsafe fn ver_priority(cache: *mut PCache, iterator: *mut VerIterator) -> i32;
		pub unsafe fn ver_uri(
			pcache: *mut PCache,
			parser: *mut PkgRecords,
			pkgfile: *mut PkgFileIterator,
		) -> *const c_char;

		// Dependency iterators
		// ====================

		pub unsafe fn ver_iter_dep_iter(iterator: *mut VerIterator) -> *mut DepIterator;
		pub unsafe fn dep_iter_release(iterator: *mut DepIterator);

		pub unsafe fn dep_iter_next(iterator: *mut DepIterator);
		pub unsafe fn dep_iter_end(iterator: *mut DepIterator) -> bool;

		// Dependency accessors
		// ====================

		// pub fn dep_iter_target_pkg(iterator: *mut DepIterator) -> *mut PkgIterator;
		// pub fn dep_iter_target_ver(iterator: *mut DepIterator) -> *const c_char;
		// pub fn dep_iter_comp_type(iterator: *mut DepIterator) -> *const c_char;
		// pub fn dep_iter_dep_type(iterator: *mut DepIterator) -> *const c_char;

		pub unsafe fn ver_file(iterator: *mut VerIterator) -> *mut VerFileIterator;
		pub unsafe fn ver_file_release(iterator: *mut VerFileIterator);

		pub unsafe fn ver_file_next(iterator: *mut VerFileIterator);
		pub unsafe fn ver_file_end(iterator: *mut VerFileIterator) -> bool;

		pub unsafe fn ver_file_lookup(
			cache: *mut PCache,
			iterator: *mut VerFileIterator,
		) -> *mut PkgRecords;
		// pub fn ver_file_parser_short_desc(parser: VerFileParser) -> *mut c_char;
		// pub fn ver_file_parser_long_desc(parser: VerFileParser) -> *mut c_char;
		pub unsafe fn long_desc(cache: *mut PCache, iterator: *mut PkgIterator) -> String;
		// pub fn ver_file_parser_maintainer(parser: VerFileParser) -> *mut c_char;
		// pub fn ver_file_parser_homepage(parser: VerFileParser) -> *mut c_char;

		pub unsafe fn ver_pkg_file(iterator: *mut VerFileIterator) -> *mut PkgFileIterator;
		pub unsafe fn pkg_file_iter_release(iterator: *mut PkgFileIterator);

		pub unsafe fn pkg_file_iter_next(iterator: *mut PkgFileIterator);
		pub unsafe fn pkg_file_iter_end(iterator: *mut PkgFileIterator) -> bool;

		pub unsafe fn pkg_file_iter_file_name(iterator: *mut PkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_archive(iterator: *mut PkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_version(iterator: *mut PkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_origin(iterator: *mut PkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_codename(iterator: *mut PkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_label(iterator: *mut PkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_site(iterator: *mut PkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_component(iterator: *mut PkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_architecture(iterator: *mut PkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_index_type(iterator: *mut PkgFileIterator) -> *const c_char;
	}
}

pub unsafe fn own_string(ptr: *const c_char) -> Option<String> {
	if ptr.is_null() {
		None
	} else {
		Some(
			ffi::CStr::from_ptr(ptr)
				.to_str()
				.expect("value should always be low-ascii")
				.to_string(),
		)
	}
}
