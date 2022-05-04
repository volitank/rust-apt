#pragma once
//#include "apt-pkg.cc"
// For Development Typing
#include "cxx-typing.h"
#include "rust/cxx.h"
//#include <memory>

struct pkgCacheFile;
struct PCache;
struct PkgIterator;
struct PkgFileIterator;
struct VerIterator;
struct VerFileIterator;
struct DepIterator;
struct VerFileParser;
struct PkgRecords;
struct PkgIndexFile;

// From Rust to C++
//
// CXX Test Function
// int greet(rust::Str greetee);


// From C++ to Rust
//
/// Main Initializers for APT
void init_config_system();
void depcache_init(PCache *pcache);

PCache *pkg_cache_create();
PkgRecords *pkg_records_create(PCache *pcache);

void pkg_cache_release(PCache *cache);
void pkg_records_release(PkgRecords *records);

int32_t pkg_cache_compare_versions(PCache *cache, const char *left, const char *right);

/// Iterator Creators
PkgIterator *pkg_begin(PCache *cache);
PkgIterator *pkg_clone(PkgIterator *iterator);
VerIterator *ver_clone(VerIterator *iterator);
VerFileIterator *ver_file(VerIterator *iterator);
VerFileIterator *ver_file_clone(VerFileIterator *iterator);

VerIterator *pkg_current_version(PkgIterator *iterator);
VerIterator *pkg_candidate_version(PCache *cache, PkgIterator *iterator);
VerIterator *pkg_version_list(PkgIterator *iterator);

PkgFileIterator *ver_pkg_file(VerFileIterator *iterator);
PkgIndexFile *pkg_index_file(PCache *pcache, PkgFileIterator *pkg_file);

PkgIterator *pkg_cache_find_name(PCache *cache, const char *name);
PkgIterator *pkg_cache_find_name_arch(PCache *cache, const char *name, const char *arch);

/// Iterator Manipulation
void pkg_next(PkgIterator *iterator);
bool pkg_end(PkgIterator *iterator);
void pkg_release(PkgIterator *iterator);

void ver_next(VerIterator *iterator);
bool ver_end(VerIterator *iterator);
void ver_release(VerIterator *iterator);

void ver_file_next(VerFileIterator *iterator);
bool ver_file_end(VerFileIterator *iterator);
void ver_file_release(VerFileIterator *iterator);

void pkg_file_release(PkgFileIterator *iterator);
void pkg_index_file_release(PkgIndexFile *wrapper);

/// Information Accessors
bool pkg_is_upgradable(PCache *cache, PkgIterator *wrapper);
bool pkg_has_versions(PkgIterator *wrapper);
bool pkg_has_provides(PkgIterator *wrapper);
rust::string get_fullname(PkgIterator *iterator, bool pretty);
const char *pkg_name(PkgIterator *iterator);
const char *pkg_arch(PkgIterator *iterator);

const char *ver_arch(VerIterator *iterator);
const char *ver_str(VerIterator *iterator);
const char *ver_section(VerIterator *iterator);
const char *ver_priority_str(VerIterator *wrapper);
const char *ver_source_package(VerIterator *iterator);
const char *ver_source_version(VerIterator *iterator);
int32_t ver_priority(PCache *pcache, VerIterator *wrapper);

/// Package Record Management
void ver_file_lookup(PkgRecords *records, VerFileIterator *iterator);
rust::string ver_uri(PkgRecords *records, PkgIndexFile *index_file);
rust::string long_desc(PCache *cache, PkgRecords *records, PkgIterator *wrapper);


/// Unused Functions
/// They may be used in the future
///
// dep_iter creation and deletion
// DepIterator *ver_iter_dep_iter(VerIterator *iterator);
// void dep_iter_release(DepIterator *iterator);

// // dep_iter mutation
// void dep_iter_next(DepIterator *iterator);
// bool dep_iter_end(DepIterator *iterator);

// // dep_iter access
// PkgIterator *dep_iter_target_pkg(DepIterator *iterator);
// const char *dep_iter_target_ver(DepIterator *iterator);
// const char *dep_iter_comp_type(DepIterator *iterator);
// const char *dep_iter_dep_type(DepIterator *iterator);

// //template<typename Iterator>
// bool validate(VerIterator *iterator, PCache *pcache);

// // ver_file_parser access
// const char *ver_file_parser_short_desc(VerFileParser *parser);
// const char *ver_file_parser_long_desc(VerFileParser *parser);

// const char *ver_file_parser_maintainer(VerFileParser *parser);
// const char *ver_file_parser_homepage(VerFileParser *parser);
// // pkg_file_iter mutation
// void pkg_file_iter_next(PkgFileIterator *iterator);
// bool pkg_file_iter_end(PkgFileIterator *iterator);

// // pkg_file_iter access
// const char *pkg_file_iter_file_name(PkgFileIterator *iterator);
// const char *pkg_file_iter_archive(PkgFileIterator *iterator);
// const char *pkg_file_iter_version(PkgFileIterator *iterator);
// const char *pkg_file_iter_origin(PkgFileIterator *iterator);
// const char *pkg_file_iter_codename(PkgFileIterator *iterator);
// const char *pkg_file_iter_label(PkgFileIterator *iterator);
// const char *pkg_file_iter_site(PkgFileIterator *iterator);
// const char *pkg_file_iter_component(PkgFileIterator *iterator);
// const char *pkg_file_iter_architecture(PkgFileIterator *iterator);
// const char *pkg_file_iter_index_type(PkgFileIterator *iterator);
