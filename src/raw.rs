use cxx::{type_id, ExternType};
/// In general:
///  * `*mut c_void` are to be released by the appropriate function
///  * `*const c_chars` are short-term borrows
///  * `*mut c_chars` are to be freed by `libc::free`.
use std::ffi;
use std::os::raw::c_char;

#[derive(Debug)]
pub struct PCache {}
unsafe impl ExternType for PCache {
	type Id = type_id!("PCache");
	type Kind = cxx::kind::Opaque;
}
pub struct PkgRecordsParser {}
unsafe impl ExternType for PkgRecordsParser {
	type Id = type_id!("PkgRecords");
	type Kind = cxx::kind::Opaque;
}
pub struct PPkgIterator {}
unsafe impl ExternType for PPkgIterator {
	type Id = type_id!("PPkgIterator");
	type Kind = cxx::kind::Opaque;
}
pub struct PVerIterator {}
unsafe impl ExternType for PVerIterator {
	type Id = type_id!("PVerIterator");
	type Kind = cxx::kind::Opaque;
}
pub struct PDepIterator {}
unsafe impl ExternType for PDepIterator {
	type Id = type_id!("PDepIterator");
	type Kind = cxx::kind::Opaque;
}
pub struct PVerFileIterator {}
unsafe impl ExternType for PVerFileIterator {
	type Id = type_id!("PVerFileIterator");
	type Kind = cxx::kind::Opaque;
}
pub struct PPkgFileIterator {}
unsafe impl ExternType for PPkgFileIterator {
	type Id = type_id!("PPkgFileIterator");
	type Kind = cxx::kind::Opaque;
}
pub struct PVerFileParser {}
unsafe impl ExternType for PVerFileParser {
	type Id = type_id!("PVerFileParser");
	type Kind = cxx::kind::Opaque;
}

#[cxx::bridge]
pub mod apt {

	unsafe extern "C++" {
		pub type PCache = crate::raw::PCache;
		pub type PkgRecords = crate::raw::PkgRecordsParser;
		pub type PPkgIterator = crate::raw::PPkgIterator;
		pub type PVerIterator = crate::raw::PVerIterator;
		pub type PDepIterator = crate::raw::PDepIterator;
		pub type PVerFileIterator = crate::raw::PVerFileIterator;
		pub type PPkgFileIterator = crate::raw::PPkgFileIterator;
		pub type PVerFileParser = crate::raw::PVerFileParser;

		include!("rust-apt/apt-pkg-c/apt-pkg.h");
		//include!("apt-pkg/cachefile.h");

		pub fn init_config_system();
		pub fn pkg_cache_create() -> *mut PCache;
		pub unsafe fn depcache_init(pcache: *mut PCache);
		// pub fn get_cache_file() -> PkgCacheFile;
		// pub fn get_cache(cache_file: PkgCacheFile) -> PkgCache;
		// pub fn get_records(cache: PkgCache) -> PkgRecords;
		// pub fn get_depcache(cache_file: PkgCacheFile) -> PkgDepCache;
		pub unsafe fn pkg_cache_release(cache: *mut PCache);

		pub unsafe fn pkg_cache_compare_versions(
			cache: *mut PCache,
			left: *const c_char,
			right: *const c_char,
		) -> i32;

		// Package iterators
		// =================

		pub unsafe fn pkg_begin(cache: *mut PCache) -> *mut PPkgIterator;
		pub unsafe fn pkg_cache_find_name(
			cache: *mut PCache,
			name: *const c_char,
		) -> *mut PPkgIterator;
		pub unsafe fn pkg_cache_find_name_arch(
			cache: *mut PCache,
			name: *const c_char,
			arch: *const c_char,
		) -> *mut PPkgIterator;
		pub unsafe fn pkg_release(iterator: *mut PPkgIterator);

		// pkgCache::PkgIterator
		pub unsafe fn pkg_next(iterator: *mut PPkgIterator);
		pub unsafe fn ver_next(iterator: *mut PVerIterator);
		pub unsafe fn pkg_end(iterator: *mut PPkgIterator) -> bool;
		pub unsafe fn ver_end(iterator: *mut PVerIterator) -> bool;

		// Package iterator accessors
		// ==========================
		pub unsafe fn pkg_clone(iterator: *mut PPkgIterator) -> *mut PPkgIterator;
		pub unsafe fn pkg_has_versions(iterator: *mut PPkgIterator) -> bool;
		pub unsafe fn pkg_has_provides(iterator: *mut PPkgIterator) -> bool;
		pub unsafe fn pkg_is_upgradable(cache: *mut PCache, iterator: *mut PPkgIterator) -> bool;
		pub unsafe fn pkg_name(iterator: *mut PPkgIterator) -> *const c_char;
		pub unsafe fn get_fullname(iterator: *mut PPkgIterator, pretty: bool) -> String;
		pub unsafe fn pkg_arch(iterator: *mut PPkgIterator) -> *const c_char;
		pub unsafe fn pkg_current_version(iterator: *mut PPkgIterator) -> *mut PVerIterator;
		pub unsafe fn pkg_candidate_version(
			cache: *mut PCache,
			iterator: *mut PPkgIterator,
		) -> *mut PVerIterator;
		pub unsafe fn validate(iterator: *mut PVerIterator, depcache: *mut PCache) -> bool;

		// Version iterators
		// =================

		pub unsafe fn pkg_version_list(pkg: *mut PPkgIterator) -> *mut PVerIterator;
		pub unsafe fn ver_release(iterator: *mut PVerIterator);

		// Version accessors
		// =================

		pub unsafe fn ver_str(iterator: *mut PVerIterator) -> *const c_char;
		pub unsafe fn ver_section(iterator: *mut PVerIterator) -> *const c_char;
		pub unsafe fn ver_source_package(iterator: *mut PVerIterator) -> *const c_char;
		pub unsafe fn ver_source_version(iterator: *mut PVerIterator) -> *const c_char;
		pub unsafe fn ver_arch(iterator: *mut PVerIterator) -> *const c_char;
		pub unsafe fn ver_priority_str(iterator: *mut PVerIterator) -> *const c_char;
		pub unsafe fn ver_priority(cache: *mut PCache, iterator: *mut PVerIterator) -> i32;
		pub unsafe fn ver_uri(
			pcache: *mut PCache,
			parser: *mut PkgRecords,
			pkgfile: *mut PPkgFileIterator,
		) -> *const c_char;

		// Dependency iterators
		// ====================

		pub unsafe fn ver_iter_dep_iter(iterator: *mut PVerIterator) -> *mut PDepIterator;
		pub unsafe fn dep_iter_release(iterator: *mut PDepIterator);

		pub unsafe fn dep_iter_next(iterator: *mut PDepIterator);
		pub unsafe fn dep_iter_end(iterator: *mut PDepIterator) -> bool;

		// Dependency accessors
		// ====================

		// pub fn dep_iter_target_pkg(iterator: *mut PDepIterator) -> *mut PPkgIterator;
		// pub fn dep_iter_target_ver(iterator: *mut PDepIterator) -> *const c_char;
		// pub fn dep_iter_comp_type(iterator: *mut PDepIterator) -> *const c_char;
		// pub fn dep_iter_dep_type(iterator: *mut PDepIterator) -> *const c_char;

		pub unsafe fn ver_file(iterator: *mut PVerIterator) -> *mut PVerFileIterator;
		pub unsafe fn ver_file_release(iterator: *mut PVerFileIterator);

		pub unsafe fn ver_file_next(iterator: *mut PVerFileIterator);
		pub unsafe fn ver_file_end(iterator: *mut PVerFileIterator) -> bool;

		pub unsafe fn ver_file_lookup(
			cache: *mut PCache,
			iterator: *mut PVerFileIterator,
		) -> *mut PkgRecords;
		// pub fn ver_file_parser_short_desc(parser: PVerFileParser) -> *mut c_char;
		// pub fn ver_file_parser_long_desc(parser: PVerFileParser) -> *mut c_char;
		pub unsafe fn long_desc(cache: *mut PCache, iterator: *mut PPkgIterator) -> String;
		// pub fn ver_file_parser_maintainer(parser: PVerFileParser) -> *mut c_char;
		// pub fn ver_file_parser_homepage(parser: PVerFileParser) -> *mut c_char;

		pub unsafe fn ver_pkg_file(iterator: *mut PVerFileIterator) -> *mut PPkgFileIterator;
		pub unsafe fn pkg_file_iter_release(iterator: *mut PPkgFileIterator);

		pub unsafe fn pkg_file_iter_next(iterator: *mut PPkgFileIterator);
		pub unsafe fn pkg_file_iter_end(iterator: *mut PPkgFileIterator) -> bool;

		pub unsafe fn pkg_file_iter_file_name(iterator: *mut PPkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_archive(iterator: *mut PPkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_version(iterator: *mut PPkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_origin(iterator: *mut PPkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_codename(iterator: *mut PPkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_label(iterator: *mut PPkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_site(iterator: *mut PPkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_component(iterator: *mut PPkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_architecture(iterator: *mut PPkgFileIterator) -> *const c_char;
		pub unsafe fn pkg_file_iter_index_type(iterator: *mut PPkgFileIterator) -> *const c_char;
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
