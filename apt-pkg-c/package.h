#pragma once
#include "rust/cxx.h"
#include <apt-pkg/cachefile.h>
#include <memory>

#include "rust-apt/src/cache.rs"

// Rust Shared Structs
struct BaseDep;
struct DepContainer;

// Apt Aliases
using DepIterator = pkgCache::DepIterator;

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

rust::Vec<DepContainer> dep_list(const VersionPtr& ver);

PackagePtr ver_parent(const VersionPtr& ver);

rust::string ver_arch(const VersionPtr& ver);

rust::string ver_str(const VersionPtr& ver);

rust::string ver_section(const VersionPtr& ver);

rust::string ver_priority_str(const VersionPtr& ver);

rust::string ver_source_name(const VersionPtr& ver);

rust::string ver_source_version(const VersionPtr& ver);

int32_t ver_priority(const std::unique_ptr<PkgCacheFile>& cache, const VersionPtr& ver);

u_int64_t ver_size(const VersionPtr& ver);

u_int64_t ver_installed_size(const VersionPtr& ver);

u_int32_t ver_id(const VersionPtr& ver);

bool ver_downloadable(const VersionPtr& ver);

bool ver_installed(const VersionPtr& ver);

rust::Vec<VersionPtr> dep_all_targets(const BaseDep& dep);
