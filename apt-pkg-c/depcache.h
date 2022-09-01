#pragma once
#include "rust/cxx.h"
#include <apt-pkg/cachefile.h>
#include <memory>

#include "rust-apt/src/cache.rs"

// Shared Rust Enum
enum class Upgrade : ::std::uint8_t;

// Apt Aliases
using PkgCacheFile = pkgCacheFile;
using PkgDepCache = pkgDepCache;

void depcache_init(const std::unique_ptr<PkgCacheFile>& cache, DynOperationProgress& callback);

void depcache_upgrade(const std::unique_ptr<PkgCacheFile>& cache,
DynOperationProgress& callback,
const Upgrade& upgrade_type);

bool pkg_is_upgradable(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool skip_depcache);

bool pkg_is_auto_installed(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

bool pkg_is_garbage(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

bool pkg_marked_install(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

bool pkg_marked_upgrade(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

bool pkg_marked_purge(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

bool pkg_marked_delete(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

bool pkg_marked_keep(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

bool pkg_marked_downgrade(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

bool pkg_marked_reinstall(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

void set_candidate_version(const std::unique_ptr<PkgCacheFile>& cache, const VersionPtr& ver);

void mark_auto(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool mark_auto);

bool mark_keep(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

bool mark_delete(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool purge);

bool mark_install(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool auto_inst, bool from_user);

void mark_reinstall(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool reinstall);

bool pkg_is_now_broken(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

bool pkg_is_inst_broken(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg);

u_int32_t install_count(const std::unique_ptr<PkgCacheFile>& cache);

u_int32_t delete_count(const std::unique_ptr<PkgCacheFile>& cache);

u_int32_t keep_count(const std::unique_ptr<PkgCacheFile>& cache);

u_int32_t broken_count(const std::unique_ptr<PkgCacheFile>& cache);

u_int64_t download_size(const std::unique_ptr<PkgCacheFile>& cache);

int64_t disk_size(const std::unique_ptr<PkgCacheFile>& cache);
