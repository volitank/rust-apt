#pragma once
#include "rust/cxx.h"
#include <apt-pkg/cachefile.h>

// Rust Shared Structs
struct PackagePtr;
struct VersionPtr;
struct VersionFile;
struct PackageFile;
struct SourceFile;
struct PackageSort;
struct DynAcquireProgress;

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


struct PkgFile {

	PkgFileIterator pkg_file;

	pkgIndexFile* index;

	PkgFile(PkgFileIterator pkg_file) : pkg_file(pkg_file), index(0){};
};

/// Main Initializers for apt:

std::unique_ptr<PkgCacheFile> pkg_cache_create();
void cache_update(const std::unique_ptr<PkgCacheFile>& cache, DynAcquireProgress& progress);

rust::Vec<SourceFile> source_uris(const std::unique_ptr<PkgCacheFile>& cache);

/// Package Functions:

rust::Vec<PackagePtr> pkg_list(
const std::unique_ptr<PkgCacheFile>& cache, const PackageSort& sort);

rust::vec<VersionFile> ver_file_list(const VersionPtr& ver);

rust::vec<PackageFile> ver_pkg_file_list(const VersionPtr& ver);

rust::vec<VersionPtr> pkg_version_list(const PackagePtr& pkg);

rust::Vec<PackagePtr> pkg_provides_list(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool cand_only);

PackagePtr pkg_cache_find_name(const std::unique_ptr<PkgCacheFile>& cache, rust::string name);

PackagePtr pkg_cache_find_name_arch(
const std::unique_ptr<PkgCacheFile>& cache, rust::string name, rust::string arch);

/// PackageFile Functions:

rust::string filename(const PackageFile& pkg_file);

rust::string archive(const PackageFile& pkg_file);

rust::string origin(const PackageFile& pkg_file);

rust::string codename(const PackageFile& pkg_file);

rust::string label(const PackageFile& pkg_file);

rust::string site(const PackageFile& pkg_file);

rust::string component(const PackageFile& pkg_file);

rust::string arch(const PackageFile& pkg_file);

rust::string index_type(const PackageFile& pkg_file);

bool pkg_file_is_trusted(const std::unique_ptr<PkgCacheFile>& cache, PackageFile& pkg_file);

u_int64_t index(const PackageFile& pkg_file);
