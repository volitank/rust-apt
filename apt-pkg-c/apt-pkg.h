#pragma once
//#include "apt-pkg.cc"
// For Development Typing
#include "cxx-typing.h"
#include "rust/cxx.h"
//#include <apt-pkg/cachefile.h>
#include <apt-pkg/pkgrecords.h>
//#include <memory>

struct pkgCacheFile;
struct PCache;
struct PPkgIterator;
struct PPkgFileIterator;
struct PVerIterator;
struct PVerFileIterator;
struct PDepIterator;
struct PVerFileParser;

using PkgRecords = pkgRecords::Parser;

// From Rust to C++
//
// CXX Test Function
// int greet(rust::Str greetee);


// From C++ to Rust
//
void init_config_system();

PCache *pkg_cache_create();
void depcache_init(PCache *pcache);

void pkg_cache_release(PCache *cache);

int32_t pkg_cache_compare_versions(PCache *cache, const char *left, const char *right);

// pkg_iter creation and deletion
PPkgIterator *pkg_begin(PCache *cache);
PPkgIterator *pkg_cache_find_name(PCache *cache, const char *name);

PPkgIterator *pkg_cache_find_name_arch(PCache *cache, const char *name, const char *arch);
PPkgIterator *pkg_clone(PPkgIterator *iterator);
void pkg_release(PPkgIterator *iterator);

// apt iterator step and check
void pkg_next(PPkgIterator *iterator);
void ver_next(PVerIterator *iterator);
bool pkg_end(PPkgIterator *iterator);
bool ver_end(PVerIterator *iterator);
const char *ver_uri(PCache *pcache, pkgRecords::Parser *parser, PPkgFileIterator *file);
// pkg_iter access

bool pkg_has_versions(PPkgIterator *wrapper);
bool pkg_has_provides(PPkgIterator *wrapper);
bool pkg_is_upgradable(PCache *cache, PPkgIterator *wrapper);
const char *pkg_name(PPkgIterator *iterator);
rust::string get_fullname(PPkgIterator *iterator, bool pretty);
const char *pkg_arch(PPkgIterator *iterator);
PVerIterator *pkg_current_version(PPkgIterator *iterator);
PVerIterator *pkg_candidate_version(PCache *cache, PPkgIterator *iterator);

// ver_iter creation and deletion
PVerIterator *pkg_version_list(PPkgIterator *iterator);
void ver_release(PVerIterator *iterator);

// ver_iter access
const char *ver_str(PVerIterator *iterator);
const char *ver_section(PVerIterator *iterator);
const char *ver_arch(PVerIterator *iterator);
const char *ver_priority_str(PVerIterator *wrapper);
const char *ver_source_package(PVerIterator *iterator);
const char *ver_source_version(PVerIterator *iterator);
int32_t ver_priority(PCache *pcache, PVerIterator *wrapper);

// dep_iter creation and deletion
PDepIterator *ver_iter_dep_iter(PVerIterator *iterator);
void dep_iter_release(PDepIterator *iterator);

// dep_iter mutation
void dep_iter_next(PDepIterator *iterator);
bool dep_iter_end(PDepIterator *iterator);

// dep_iter access
PPkgIterator *dep_iter_target_pkg(PDepIterator *iterator);
const char *dep_iter_target_ver(PDepIterator *iterator);
const char *dep_iter_comp_type(PDepIterator *iterator);
const char *dep_iter_dep_type(PDepIterator *iterator);

// ver_file_iter creation and deletion
PVerFileIterator *ver_file(PVerIterator *iterator);
void ver_file_release(PVerFileIterator *iterator);

// ver_file_iter mutation
void ver_file_next(PVerFileIterator *iterator);
bool ver_file_end(PVerFileIterator *iterator);
//template<typename Iterator>
bool validate(PVerIterator *iterator, PCache *pcache);

// ver_file_parser creation
pkgRecords::Parser *ver_file_lookup(PCache *pcache, PVerFileIterator *iterator);

// ver_file_parser access
// const char *ver_file_parser_short_desc(PVerFileParser *parser);
// const char *ver_file_parser_long_desc(PVerFileParser *parser);
rust::string long_desc(PCache *cache, PPkgIterator *wrapper);
// const char *ver_file_parser_maintainer(PVerFileParser *parser);
// const char *ver_file_parser_homepage(PVerFileParser *parser);

// ver_file_iter has no accessors, only the creation of pkg_file_iter


// pkg_file_iter creation
PPkgFileIterator *ver_pkg_file(PVerFileIterator *iterator);
void pkg_file_iter_release(PPkgFileIterator *iterator);

// pkg_file_iter mutation
void pkg_file_iter_next(PPkgFileIterator *iterator);
bool pkg_file_iter_end(PPkgFileIterator *iterator);

// pkg_file_iter access
const char *pkg_file_iter_file_name(PPkgFileIterator *iterator);
const char *pkg_file_iter_archive(PPkgFileIterator *iterator);
const char *pkg_file_iter_version(PPkgFileIterator *iterator);
const char *pkg_file_iter_origin(PPkgFileIterator *iterator);
const char *pkg_file_iter_codename(PPkgFileIterator *iterator);
const char *pkg_file_iter_label(PPkgFileIterator *iterator);
const char *pkg_file_iter_site(PPkgFileIterator *iterator);
const char *pkg_file_iter_component(PPkgFileIterator *iterator);
const char *pkg_file_iter_architecture(PPkgFileIterator *iterator);
const char *pkg_file_iter_index_type(PPkgFileIterator *iterator);
