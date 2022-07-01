#pragma once
#include "rust/cxx.h"
#include <apt-pkg/cachefile.h>
#include <apt-pkg/pkgrecords.h>

struct PkgRecords {

	pkgRecords records;
	// Parser doesn't want to work as a UniquePtr
	pkgRecords::Parser* parser;

	u_int32_t last;

	PkgRecords(pkgCache* cache) : records(*cache), last(0){};
};

// Rust Shared Structs
struct Records;
struct PackagePtr;
struct VersionPtr;
struct PackageFile;
struct DepContainer;
struct BaseDep;
struct SourceFile;
struct PackageSort;
struct DynUpdateProgress;

// Apt Aliases
using PkgCacheFile = pkgCacheFile;
using PkgCache = pkgCache;
using PkgSourceList = pkgSourceList;
using PkgDepCache = pkgDepCache;

using PkgIterator = pkgCache::PkgIterator;
using VerIterator = pkgCache::VerIterator;
using VerFileIterator = pkgCache::VerFileIterator;
using PkgFileIterator = pkgCache::PkgFileIterator;
using DescIterator = pkgCache::DescIterator;
using DepIterator = pkgCache::DepIterator;


/// Main Initializers for apt:

std::unique_ptr<PkgCacheFile> pkg_cache_create();
void cache_update(const std::unique_ptr<PkgCacheFile>& cache, DynUpdateProgress& progress);

Records pkg_records_create(const std::unique_ptr<PkgCacheFile>& cache);
std::unique_ptr<PkgDepCache> depcache_create(const std::unique_ptr<PkgCacheFile>& cache);

rust::Vec<SourceFile> source_uris(const std::unique_ptr<PkgCacheFile>& cache);
int32_t pkg_cache_compare_versions(
const std::unique_ptr<PkgCacheFile>& cache, const char* left, const char* right);

/// Package Functions:

rust::Vec<PackagePtr> pkg_list(
const std::unique_ptr<PkgCacheFile>& cache, const PackageSort& sort);
rust::Vec<PackagePtr> pkg_provides_list(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool cand_only);
rust::vec<VersionPtr> pkg_version_list(const PackagePtr& pkg);

PackagePtr pkg_cache_find_name(const std::unique_ptr<PkgCacheFile>& cache, rust::string name);
PackagePtr pkg_cache_find_name_arch(
const std::unique_ptr<PkgCacheFile>& cache, rust::string name, rust::string arch);
VersionPtr pkg_current_version(const PackagePtr& pkg);
VersionPtr pkg_candidate_version(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

bool pkg_is_installed(const PackagePtr& pkg);
bool pkg_has_versions(const PackagePtr& pkg);
bool pkg_has_provides(const PackagePtr& pkg);
bool pkg_essential(const PackagePtr& pkg);
rust::string get_fullname(const PackagePtr& pkg, bool pretty);
rust::string pkg_name(const PackagePtr& pkg);
rust::string pkg_arch(const PackagePtr& pkg);
u_int32_t pkg_id(const PackagePtr& pkg);
u_int8_t pkg_current_state(const PackagePtr& pkg);
u_int8_t pkg_inst_state(const PackagePtr& pkg);
u_int8_t pkg_selected_state(const PackagePtr& pkg);

/// Version Functions:

rust::vec<PackageFile> pkg_file_list(
const std::unique_ptr<PkgCacheFile>& cache, const VersionPtr& ver);
rust::Vec<DepContainer> dep_list(const VersionPtr& ver);

rust::string ver_arch(const VersionPtr& ver);
rust::string ver_str(const VersionPtr& ver);
rust::string ver_section(const VersionPtr& ver);
rust::string ver_priority_str(const VersionPtr& ver);
rust::string ver_source_name(const VersionPtr& ver);
rust::string ver_source_version(const VersionPtr& ver);
rust::string ver_name(const VersionPtr& ver);
int32_t ver_priority(const std::unique_ptr<PkgCacheFile>& cache, const VersionPtr& ver);
u_int64_t ver_size(const VersionPtr& ver);
u_int64_t ver_installed_size(const VersionPtr& ver);
u_int32_t ver_id(const VersionPtr& ver);
bool ver_downloadable(const VersionPtr& ver);
bool ver_installed(const VersionPtr& ver);

/// DepCache Information Accessors:

bool pkg_is_upgradable(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool skip_depcache);
bool pkg_is_auto_installed(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);
bool pkg_is_garbage(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);
bool pkg_marked_install(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);
bool pkg_marked_upgrade(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);
bool pkg_marked_delete(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);
bool pkg_marked_keep(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);
bool pkg_marked_downgrade(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);
bool pkg_marked_reinstall(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);
bool pkg_is_now_broken(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);
bool pkg_is_inst_broken(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

u_int32_t install_count(const std::unique_ptr<PkgCacheFile>& cache);
u_int32_t delete_count(const std::unique_ptr<PkgCacheFile>& cache);
u_int32_t keep_count(const std::unique_ptr<PkgCacheFile>& cache);
u_int32_t broken_count(const std::unique_ptr<PkgCacheFile>& cache);
u_int64_t download_size(const std::unique_ptr<PkgCacheFile>& cache);
int64_t disk_size(const std::unique_ptr<PkgCacheFile>& cache);

/// Package Record Management:

void ver_file_lookup(Records& records, const PackageFile& pkg_file);
void desc_file_lookup(Records& records, const std::unique_ptr<DescIterator>& desc);
rust::string ver_uri(const Records& records,
const std::unique_ptr<PkgCacheFile>& cache,
const PackageFile& pkg_file);
rust::string long_desc(const Records& records);
rust::string short_desc(const Records& records);
rust::string hash_find(const Records& records, rust::string hash_type);

rust::Vec<VersionPtr> dep_all_targets(const BaseDep& dep);
