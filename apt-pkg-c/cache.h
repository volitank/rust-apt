#pragma once
#include "rust/cxx.h"
#include <apt-pkg/cachefile.h>

// Rust Shared Structs
struct PackagePtr;
struct VersionPtr;
struct PackageFile;
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

rust::Vec<SourceFile> source_uris(const std::unique_ptr<PkgCacheFile>& cache);

/// Package Functions:

rust::Vec<PackagePtr> pkg_list(
const std::unique_ptr<PkgCacheFile>& cache, const PackageSort& sort);

rust::vec<PackageFile> pkg_file_list(
const std::unique_ptr<PkgCacheFile>& cache, const VersionPtr& ver);

rust::vec<VersionPtr> pkg_version_list(const PackagePtr& pkg);

rust::Vec<PackagePtr> pkg_provides_list(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool cand_only);

PackagePtr pkg_cache_find_name(const std::unique_ptr<PkgCacheFile>& cache, rust::string name);
PackagePtr pkg_cache_find_name_arch(
const std::unique_ptr<PkgCacheFile>& cache, rust::string name, rust::string arch);
