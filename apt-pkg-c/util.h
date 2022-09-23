#pragma once
#include "rust/cxx.h"
#include <cstdint>

#include "rust-apt/src/cache.rs"

/// Internal Helper Functions.
/// Do not expose these on the Rust side - only for use on the C++ side.
///
/// Handle any apt errors and return result to rust.
void handle_errors();

/// Handle the situation where a string is null and return a result to rust
rust::string handle_null_str(const char* str);

/// Wrap the PkgIterator into our PackagePtr Struct.
/// Return Result if it's null
PackagePtr wrap_package(pkgCache::PkgIterator pkg);

/// Wrap the VerIterator into our VersionPtr Struct.
/// Return Result if it's null
VersionPtr wrap_version(pkgCache::VerIterator ver);

/// Wrap PkgFileIterator into PackageFile Struct.
PackageFile wrap_pkg_file(pkgCache::PkgFileIterator pkg_file);

/// Wrap VerFileIterator into VersionFile Struct.
VersionFile wrap_ver_file(pkgCache::VerFileIterator ver_file);

/// Determine if the package is upgradable without the depcache.
bool is_upgradable(
const std::unique_ptr<PkgCacheFile>& cache, const pkgCache::PkgIterator& pkg);

/// Determine if the package is auto removable.
bool is_auto_removable(
const std::unique_ptr<PkgCacheFile>& cache, const pkgCache::PkgIterator& pkg);

/// Determine if the package is auto installed.
bool is_auto_installed(
const std::unique_ptr<PkgCacheFile>& cache, const pkgCache::PkgIterator& pkg);

//////////////////////////////////
/// End Internal Helper Functions.
//////////////////////////////////

/// Compare two package version strings.
int32_t cmp_versions(rust::String ver1_rust, rust::String ver2_rust);

/// Return an APT-styled progress bar (`[####  ]`).
rust::String get_apt_progress_string(float percent, uint32_t output_width);

/// Lock the APT lockfile.
void apt_lock();

/// Unock the APT lockfile.
void apt_unlock();

/// Lock the Dpkg lockfile.
void apt_lock_inner();

/// Unlock the Dpkg lockfile.
void apt_unlock_inner();

/// Check if the lockfile is locked.
bool apt_is_locked();
