#include <apt-pkg/algorithms.h>
#include <apt-pkg/policy.h>

#include "rust-apt/src/depcache.rs"

static bool is_upgradable(
const std::unique_ptr<PkgCacheFile>& cache, const pkgCache::PkgIterator& pkg) {
	pkgCache::VerIterator inst = pkg.CurrentVer();
	if (!inst) return false;

	pkgCache::VerIterator cand = cache->GetPolicy()->GetCandidateVer(pkg);
	if (!cand) return false;

	return inst != cand;
}

/// Create the depcache.
std::unique_ptr<PkgDepCache> depcache_create(const std::unique_ptr<PkgCacheFile>& cache) {
	pkgApplyStatus(*cache->GetDepCache());
	return std::make_unique<pkgDepCache>(*cache->GetDepCache());
}

/// Is the Package upgradable?
///
/// `skip_depcache = true` increases performance by skipping the pkgDepCache
/// Skipping the depcache is very unnecessary if it's already been initialized
/// If you're not sure, set `skip_depcache = false`
bool pkg_is_upgradable(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool skip_depcache) {
	if (!pkg.ptr->CurrentVer()) {
		return false;
	}
	if (skip_depcache) return is_upgradable(cache, *pkg.ptr);
	return (*cache->GetDepCache())[*pkg.ptr].Upgradable();
}


/// Is the Package auto installed? Packages marked as auto installed are usually dependencies.
bool pkg_is_auto_installed(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	pkgDepCache::StateCache state = (*cache->GetDepCache())[*pkg.ptr];
	return state.Flags & pkgCache::Flag::Auto;
}


/// Is the Package able to be auto removed?
bool pkg_is_garbage(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].Garbage;
}


/// Is the Package marked for install?
bool pkg_marked_install(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].NewInstall();
}


/// Is the Package marked for upgrade?
bool pkg_marked_upgrade(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].Upgrade();
}


/// Is the Package marked for removal?
bool pkg_marked_delete(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].Delete();
}


/// Is the Package marked for keep?
bool pkg_marked_keep(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].Keep();
}


/// Is the Package marked for downgrade?
bool pkg_marked_downgrade(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].Downgrade();
}


/// Is the Package marked for reinstall?
bool pkg_marked_reinstall(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].ReInstall();
}


/// Is the installed Package broken?
bool pkg_is_now_broken(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].NowBroken();
}


/// Is the Package to be installed broken?
bool pkg_is_inst_broken(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].InstBroken();
}


/// The number of packages marked for installation.
u_int32_t install_count(const std::unique_ptr<PkgCacheFile>& cache) {
	return cache->GetDepCache()->InstCount();
}


/// The number of packages marked for removal.
u_int32_t delete_count(const std::unique_ptr<PkgCacheFile>& cache) {
	return cache->GetDepCache()->DelCount();
}


/// The number of packages marked for keep.
u_int32_t keep_count(const std::unique_ptr<PkgCacheFile>& cache) {
	return cache->GetDepCache()->KeepCount();
}


/// The number of packages with broken dependencies in the cache.
u_int32_t broken_count(const std::unique_ptr<PkgCacheFile>& cache) {
	return cache->GetDepCache()->BrokenCount();
}


/// The size of all packages to be downloaded.
u_int64_t download_size(const std::unique_ptr<PkgCacheFile>& cache) {
	return cache->GetDepCache()->DebSize();
}


/// The amount of space required for installing/removing the packages,"
///
/// i.e. the Installed-Size of all packages marked for installation"
/// minus the Installed-Size of all packages for removal."
int64_t disk_size(const std::unique_ptr<PkgCacheFile>& cache) {
	return cache->GetDepCache()->UsrSize();
}
