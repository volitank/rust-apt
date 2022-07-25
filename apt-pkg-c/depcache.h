#pragma once
#include "rust/cxx.h"
#include <apt-pkg/cachefile.h>
#include <memory>

#include "rust-apt/src/cache.rs"

// Apt Aliases
using PkgCacheFile = pkgCacheFile;
using PkgDepCache = pkgDepCache;

std::unique_ptr<PkgDepCache> depcache_create(const std::unique_ptr<PkgCacheFile>& cache);

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
