#pragma once
#include "rust/cxx.h"
#include <apt-pkg/cachefile.h>
#include <apt-pkg/upgrade.h>
#include <memory>

#include "rust-apt/src/raw/depcache.rs"

/// Clear any marked changes in the DepCache.
inline void DepCache::init(DynOperationProgress& callback) const {
	OpProgressWrapper op_progress(callback);

	(*ptr)->Init(&op_progress);
	// pkgApplyStatus(*cache->GetDepCache());
	handle_errors();
}

/// Autoinstall every broken package and run the problem resolver
/// Returns false if the problem resolver fails.
inline bool DepCache::fix_broken() const noexcept {
	return pkgFixBroken(**ptr);
}

inline ActionGroup DepCache::action_group() const noexcept {
	return ActionGroup{ std::make_unique<PkgActionGroup>(**ptr) };
}

inline void ActionGroup::release() const noexcept { ptr->release(); }

/// Is the Package upgradable?
///
/// `skip_depcache = true` increases performance by skipping the pkgDepCache
/// Skipping the depcache is very unnecessary if it's already been
/// initialized If you're not sure, set `skip_depcache = false`
inline bool DepCache::is_upgradable(const Package& pkg) const noexcept {
	return (**ptr)[*pkg.ptr].Upgradable();
}

/// Is the Package auto installed? Packages marked as auto installed are usually dependencies.
inline bool DepCache::is_auto_installed(const Package& pkg) const noexcept {
	pkgDepCache::StateCache state = (**ptr)[*pkg.ptr];
	return state.Flags & pkgCache::Flag::Auto;
}

/// Is the Package able to be auto removed?
inline bool DepCache::is_garbage(const Package& pkg) const noexcept {
	return (**ptr)[*pkg.ptr].Garbage;
}

/// Is the Package marked for install?
inline bool DepCache::marked_install(const Package& pkg) const noexcept {
	return (**ptr)[*pkg.ptr].NewInstall();
}

/// Is the Package marked for upgrade?
inline bool DepCache::marked_upgrade(const Package& pkg) const noexcept {
	return (**ptr)[*pkg.ptr].Upgrade();
}

/// Is the Package marked to be purged?
inline bool DepCache::marked_purge(const Package& pkg) const noexcept {
	return (**ptr)[*pkg.ptr].Purge();
}

/// Is the Package marked for removal?
inline bool DepCache::marked_delete(const Package& pkg) const noexcept {
	return (**ptr)[*pkg.ptr].Delete();
}

/// Is the Package marked for keep?
inline bool DepCache::marked_keep(const Package& pkg) const noexcept {
	return (**ptr)[*pkg.ptr].Keep();
}

/// Is the Package marked for downgrade?
inline bool DepCache::marked_downgrade(const Package& pkg) const noexcept {
	return (**ptr)[*pkg.ptr].Downgrade();
}

/// Is the Package marked for reinstall?
inline bool DepCache::marked_reinstall(const Package& pkg) const noexcept {
	return (**ptr)[*pkg.ptr].ReInstall();
}

/// Mark a package as automatically installed.
///
/// MarkAuto = true will mark the package as automatically installed and false will mark it as manual
inline void DepCache::mark_auto(const Package& pkg, bool mark_auto) const noexcept {
	(*ptr)->MarkAuto(*pkg.ptr, mark_auto);
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
///     and the bool is passed from `MarkInstall` itselfconst .
///     I don't believe anyone needs access to this boolconst .
///
/// Depth:
///     Recursion tracker and is only used for printing Debug statements.
///     No one needs access to this. Additionally Depth cannot be over 3000.
inline bool DepCache::mark_keep(const Package& pkg) const noexcept {
	return (*ptr)->MarkKeep(*pkg.ptr, false, false);
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
inline bool DepCache::mark_delete(const Package& pkg, bool purge) const noexcept {
	return (*ptr)->MarkDelete(*pkg.ptr, purge);
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
inline bool DepCache::mark_install(
const Package& pkg, bool auto_inst, bool from_user) const noexcept {
	return (*ptr)->MarkInstall(*pkg.ptr, auto_inst, 0, from_user, false);
}

/// Set a version to be the candidate of it's package.
inline void DepCache::set_candidate_version(const Version& ver) const noexcept {
	(*ptr)->SetCandidateVersion(*ver.ptr);
}

/// Return the candidate version of the package.
/// Ptr will be NULL if there isn't a candidate.
inline Version DepCache::unsafe_candidate_version(const Package& pkg) const noexcept {
	return Version{ std::make_unique<VerIterator>(
	(*ptr)->GetCandidateVersion(*pkg.ptr)) };
}

/// Mark a package for reinstallation
///
/// To:
///     True = The package will be marked for reinstall
///     False = The package will be unmarked for reinstall
inline void DepCache::mark_reinstall(const Package& pkg, bool reinstall) const noexcept {
	(*ptr)->SetReInstall(*pkg.ptr, reinstall);
}

/// Is the installed Package broken?
inline bool DepCache::is_now_broken(const Package& pkg) const noexcept {
	return (**ptr)[*pkg.ptr].NowBroken();
}

/// Is the Package to be installed broken?
inline bool DepCache::is_inst_broken(const Package& pkg) const noexcept {
	return (**ptr)[*pkg.ptr].InstBroken();
}

/// The number of packages marked for installation.
inline u_int32_t DepCache::install_count() const noexcept {
	return (*ptr)->InstCount();
}

/// The number of packages marked for removal.
inline u_int32_t DepCache::delete_count() const noexcept {
	return (*ptr)->DelCount();
}

/// The number of packages marked for keep.
inline u_int32_t DepCache::keep_count() const noexcept {
	return (*ptr)->KeepCount();
}

/// The number of packages with broken dependencies in the cache.
inline u_int32_t DepCache::broken_count() const noexcept {
	return (*ptr)->BrokenCount();
}

/// The size of all packages to be downloaded.
inline u_int64_t DepCache::download_size() const noexcept {
	return (*ptr)->DebSize();
}

/// The amount of space required for installing/removing the packages,"
///
/// i.e. the Installed-Size of all packages marked for installation"
/// minus the Installed-Size of all packages for removal."
inline int64_t DepCache::disk_size() const noexcept {
	return (*ptr)->UsrSize();
}

/// Perform a Full Upgrade. Remove and install new packages if necessary.
inline void DepCache::full_upgrade(DynOperationProgress& callback) const {
	OpProgressWrapper op_progress(callback);

	// This is equivalent to `apt full-upgrade` and `apt-get dist-upgrade`
	// It is currently unclear if we should return a bool here. I think Result should be fine.
	APT::Upgrade::Upgrade(**ptr, APT::Upgrade::ALLOW_EVERYTHING, &op_progress);
	handle_errors();
}

/// Perform a Safe Upgrade. Neither remove or install new packages.
inline void DepCache::safe_upgrade(DynOperationProgress& callback) const {
	OpProgressWrapper op_progress(callback);

	// This is equivalent to `apt-get upgrade`
	APT::Upgrade::Upgrade(**ptr,
	APT::Upgrade::FORBID_REMOVE_PACKAGES | APT::Upgrade::FORBID_INSTALL_NEW_PACKAGES,
	&op_progress);
	handle_errors();
}

/// Perform an Install Upgrade. New packages will be installed but nothing will be removed.
inline void DepCache::install_upgrade(DynOperationProgress& callback) const {
	OpProgressWrapper op_progress(callback);

	// This is equivalent to `apt upgrade`
	APT::Upgrade::Upgrade(**ptr, APT::Upgrade::FORBID_REMOVE_PACKAGES, &op_progress);
	handle_errors();
}
