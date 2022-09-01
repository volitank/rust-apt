#include <apt-pkg/algorithms.h>
#include <apt-pkg/policy.h>
#include <apt-pkg/upgrade.h>

#include "rust-apt/src/depcache.rs"

/// Helper Functions:

/// Handle any apt errors and return result to rust.
static void handle_errors() {
	std::string err_str;
	while (!_error->empty()) {
		std::string msg;
		bool Type = _error->PopMessage(msg);
		err_str.append(Type == true ? "E:" : "W:");
		err_str.append(msg);
		err_str.append(";");
	}

	// Throwing runtime_error returns result to rust.
	// Remove the last ";" in the string before sending it.
	if (err_str.length()) {
		err_str.pop_back();
		throw std::runtime_error(err_str);
	}
}

static bool is_upgradable(
const std::unique_ptr<PkgCacheFile>& cache, const pkgCache::PkgIterator& pkg) {
	pkgCache::VerIterator inst = pkg.CurrentVer();
	if (!inst) return false;

	pkgCache::VerIterator cand = cache->GetPolicy()->GetCandidateVer(pkg);
	if (!cand) return false;

	return inst != cand;
}

/// Clear any marked changes in the DepCache.
void depcache_init(const std::unique_ptr<PkgCacheFile>& cache, DynOperationProgress& callback) {
	OpProgressWrapper op_progress(callback);
	cache->GetDepCache()->Init(&op_progress);
	// pkgApplyStatus(*cache->GetDepCache());
	handle_errors();
}

/// Upgrade the depcache
void depcache_upgrade(const std::unique_ptr<PkgCacheFile>& cache,
DynOperationProgress& callback,
const Upgrade& upgrade_type) {
	// Apt Upgrade Enum
	// APT::Upgrade::ALLOW_EVERYTHING;
	// APT::Upgrade::FORBID_REMOVE_PACKAGES;
	// APT::Upgrade::FORBID_INSTALL_NEW_PACKAGES;

	OpProgressWrapper op_progress(callback);
	bool ret;

	// This is equivalent to `apt full-upgrade` and `apt-get dist-upgrade`
	if (upgrade_type == Upgrade::FullUpgrade) {
		ret = APT::Upgrade::Upgrade(
		*cache->GetDepCache(), APT::Upgrade::ALLOW_EVERYTHING, &op_progress);

		// This is equivalent to `apt-get upgrade`
	} else if (upgrade_type == Upgrade::SafeUpgrade) {
		ret = APT::Upgrade::Upgrade(*cache->GetDepCache(),
		APT::Upgrade::FORBID_REMOVE_PACKAGES | APT::Upgrade::FORBID_INSTALL_NEW_PACKAGES,
		&op_progress);

		// This is equivalent to `apt upgrade`
		// Upgrade::Upgrade
	} else {
		ret = APT::Upgrade::Upgrade(*cache->GetDepCache(),
		APT::Upgrade::FORBID_REMOVE_PACKAGES, &op_progress);
	}

	// Handle any errors in the event Upgrade returns false.
	if (!ret) {
		handle_errors();
	}
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


/// Is the Package marked to be purged?
bool pkg_marked_purge(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].Purge();
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

/// Mark a package as automatically installed.
///
/// MarkAuto = true will mark the package as automatically installed and false will mark it as manual
void mark_auto(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool mark_auto) {
	cache->GetDepCache()->MarkAuto(*pkg.ptr, mark_auto);
}

/// Mark a package for keep.
///
///     This means that the package will not be changed from its current version.
///     This will not stop a reinstall, but will stop removal, upgrades and downgrades
///
/// Soft:
///     True = will mark for keep
///     False = will unmark for keep
///
///     We don't believe that there is any reason to unmark packages for keep.
///     If someone has a reason, and would like it implemented, please put in a feature request.
///
/// FromUser:
///     This is only ever True in apt underneath `MarkInstall`,
///     and the bool is passed from `MarkInstall` itself.
///     I don't believe anyone needs access to this bool.
///
/// Depth:
///     Recursion tracker and is only used for printing Debug statements.
///     No one needs access to this. Additionally Depth cannot be over 3000.
bool mark_keep(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return cache->GetDepCache()->MarkKeep(*pkg.ptr, false, false);
}

/// Mark a package for removal.
///
/// MarkPurge:
///     True the package will be purged.
///     False the package will not be purged.
///
/// Depth:
///     Recursion tracker and is only used for printing Debug statements.
///     No one needs access to this. Additionally Depth cannot be over 3000.
///
/// FromUser:
///     True if the user requested this.
///     False the User did not request this.
///
///     Typically You would always use from user.
///     False here appears to be more of an implementation detail.
bool mark_delete(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool purge) {
	return cache->GetDepCache()->MarkDelete(*pkg.ptr, purge);
}

/// Mark a package for installation.
///
/// AutoInst: true = Auto Install dependencies of the package.
///
/// FromUser: true = Mark the package as installed from the User.
///
/// Depth:
///     Recursion tracker and is only used for printing Debug statements.
///     No one needs access to this. Additionally Depth cannot be over 3000.
///
/// ForceImportantDeps = TODO: Study what this does.
/// TODO: Maybe make a separate function on the higher level `mark_install_with_deps`
/// TODO: and hide the auto_inst option. Alternatively an enum could be passed that would dictate
/// TODO: If auto_inst, from_user or both will be true. Not sure which is most intuitive
bool mark_install(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool auto_inst, bool from_user) {
	return cache->GetDepCache()->MarkInstall(*pkg.ptr, auto_inst, 0, from_user, false);
}

/// Set a version to be the candidate of it's package.
void set_candidate_version(const std::unique_ptr<PkgCacheFile>& cache, const VersionPtr& ver) {
	cache->GetDepCache()->SetCandidateVersion(*ver.ptr);
}

/// Mark a package for reinstallation
///
/// To:
///     True = The package will be marked for reinstall
///     False = The package will be unmarked for reinstall
void mark_reinstall(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool reinstall) {
	cache->GetDepCache()->SetReInstall(*pkg.ptr, reinstall);
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
