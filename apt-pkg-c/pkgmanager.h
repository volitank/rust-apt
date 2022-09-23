#pragma once
#include <apt-pkg/cachefile.h>
#include <apt-pkg/dpkgpm.h>
#include <apt-pkg/packagemanager.h>
#include <memory>

#include "rust-apt/src/cache.rs"
#include "rust-apt/src/progress.rs"
#include "rust-apt/src/records.rs"

using PkgPackageManager = pkgPackageManager;

std::unique_ptr<PkgPackageManager> pkgmanager_create(
const std::unique_ptr<PkgCacheFile>& cache);

void pkgmanager_get_archives(const std::unique_ptr<PkgPackageManager>& pkgmanager,
const std::unique_ptr<PkgCacheFile>& cache,
Records& records,
DynAcquireProgress& callback);

void pkgmanager_do_install(const std::unique_ptr<PkgPackageManager>& pkgmanager,
DynInstallProgress& callback);
