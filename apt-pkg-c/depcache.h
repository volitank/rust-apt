#pragma once
#include <apt-pkg/cachefile.h>
#include <apt-pkg/upgrade.h>
#include <memory>
#include "package.h"
#include "progress.h"
#include "util.h"

using ActionGroup = pkgDepCache::ActionGroup;

struct PkgDepCache {
	pkgDepCache* ptr;

	// Maybe we use this if we don't want pin_mut() all over the place in Rust.
	PkgDepCache* unconst() const { return const_cast<PkgDepCache*>(this); }

	std::unique_ptr<ActionGroup> action_group() const {
		return std::make_unique<ActionGroup>(*ptr);
	}

	bool is_upgradable(const PkgIterator& pkg) const { return (*ptr)[pkg].Upgradable(); }

	bool fix_broken() const { return pkgFixBroken(*ptr); }

	/// Is the Package auto installed? Packages marked as auto installed are usually dependencies.
	bool is_auto_installed(const PkgIterator& pkg) const {
		pkgDepCache::StateCache state = (*ptr)[pkg];
		return state.Flags & pkgCache::Flag::Auto;
	}

	/// Is the Package able to be auto removed?
	bool is_garbage(const PkgIterator& pkg) const { return (*ptr)[pkg].Garbage; }

	/// Is the Package marked for install?
	bool marked_install(const PkgIterator& pkg) const { return (*ptr)[pkg].NewInstall(); }

	/// Is the Package marked for upgrade?
	bool marked_upgrade(const PkgIterator& pkg) const { return (*ptr)[pkg].Upgrade(); }

	/// Is the Package marked to be purged?
	bool marked_purge(const PkgIterator& pkg) const { return (*ptr)[pkg].Purge(); }

	/// Is the Package marked for removal?
	bool marked_delete(const PkgIterator& pkg) const { return (*ptr)[pkg].Delete(); }

	/// Is the Package marked for keep?
	bool marked_keep(const PkgIterator& pkg) const { return (*ptr)[pkg].Keep(); }

	/// Is the Package marked for downgrade?
	bool marked_downgrade(const PkgIterator& pkg) const { return (*ptr)[pkg].Downgrade(); }

	/// Is the Package marked for reinstall?
	bool marked_reinstall(const PkgIterator& pkg) const { return (*ptr)[pkg].ReInstall(); }

	/// Mark a package as automatically installed.
	///
	/// MarkAuto = true will mark the package as automatically installed and false will mark it as
	/// manual
	void mark_auto(const PkgIterator& pkg, bool mark_auto) const { ptr->MarkAuto(pkg, mark_auto); }

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
	bool mark_keep(const PkgIterator& pkg) const { return ptr->MarkKeep(pkg, false, false); }

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
	bool mark_delete(const PkgIterator& pkg, bool purge) const {
		return ptr->MarkDelete(pkg, purge);
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
	bool mark_install(const PkgIterator& pkg, bool auto_inst, bool from_user) const {
		return ptr->MarkInstall(pkg, auto_inst, 0, from_user, false);
	}

	/// Set a version to be the candidate of it's package.
	void set_candidate_version(const VerIterator& ver) const { ptr->SetCandidateVersion(ver); }

	/// Return the candidate version of the package.
	std::unique_ptr<VerIterator> u_candidate_version(const PkgIterator& pkg) const {
		return std::make_unique<VerIterator>(ptr->GetCandidateVersion(pkg));
	}

	/// Returns the installed version if it exists.
	/// * If a version is marked for install this will return the version to be
	///   installed.
	/// * If an installed package is marked for removal, this will return [`None`].
	std::unique_ptr<VerIterator> u_install_version(const PkgIterator& pkg) const {
		pkgCache& cache = ptr->GetCache();

		return std::make_unique<VerIterator>((*ptr)[pkg].InstVerIter(cache));
	}

	/// Returns the state of the dependency as u8
	uint8_t dep_state(const DepIterator& dep) const { return (*ptr)[dep]; }

	/// Checks if the dependency is important.
	///
	/// Depends, PreDepends, Conflicts, Obsoletes, Breaks
	/// will return [true].
	///
	/// Suggests, Recommends will return [true] if they are
	/// configured to be installed.
	bool is_important_dep(const DepIterator& dep) const { return ptr->IsImportantDep(dep); }

	/// Mark a package for reinstallation
	///
	/// To:
	///     True = The package will be marked for reinstall
	///     False = The package will be unmarked for reinstall
	void mark_reinstall(const PkgIterator& pkg, bool reinstall) const {
		ptr->SetReInstall(pkg, reinstall);
	}

	/// Is the installed Package broken?
	bool is_now_broken(const PkgIterator& pkg) const { return (*ptr)[pkg].NowBroken(); }

	/// Is the Package to be installed broken?
	bool is_inst_broken(const PkgIterator& pkg) const { return (*ptr)[pkg].InstBroken(); }

	/// The number of packages marked for installation.
	u_int32_t install_count() const { return ptr->InstCount(); }

	/// The number of packages marked for removal.
	u_int32_t delete_count() const { return ptr->DelCount(); }

	/// The number of packages marked for keep.
	u_int32_t keep_count() const { return ptr->KeepCount(); }

	/// The number of packages with broken dependencies in the cache.
	u_int32_t broken_count() const { return ptr->BrokenCount(); }

	/// The size of all packages to be downloaded.
	u_int64_t download_size() const { return ptr->DebSize(); }

	/// The amount of space required for installing/removing the packages,"
	///
	/// i.e. the Installed-Size of all packages marked for installation"
	/// minus the Installed-Size of all packages for removal."
	int64_t disk_size() const { return ptr->UsrSize(); }

	/// Perform a Full Upgrade. Remove and install new packages if necessary.
	void u_full_upgrade(DynOperationProgress& callback) const {
		OpProgressWrapper op_progress(callback);

		// This is equivalent to `apt full-upgrade` and `apt-get dist-upgrade`
		// It is currently unclear if we should return a bool here. I think Result should be fine.
		APT::Upgrade::Upgrade(*ptr, APT::Upgrade::ALLOW_EVERYTHING, &op_progress);
		handle_errors();
	}

	/// Perform a Safe Upgrade. Neither remove or install new packages.
	void u_safe_upgrade(DynOperationProgress& callback) const {
		OpProgressWrapper op_progress(callback);

		// This is equivalent to `apt-get upgrade`
		APT::Upgrade::Upgrade(
			*ptr, APT::Upgrade::FORBID_REMOVE_PACKAGES | APT::Upgrade::FORBID_INSTALL_NEW_PACKAGES,
			&op_progress
		);
		handle_errors();
	}

	/// Perform an Install Upgrade. New packages will be installed but nothing will be removed.
	void u_install_upgrade(DynOperationProgress& callback) const {
		OpProgressWrapper op_progress(callback);

		// This is equivalent to `apt upgrade`
		APT::Upgrade::Upgrade(*ptr, APT::Upgrade::FORBID_REMOVE_PACKAGES, &op_progress);
		handle_errors();
	}

	/// Clear any marked changes in the DepCache.
	void u_init(DynOperationProgress& callback) const {
		OpProgressWrapper op_progress(callback);

		ptr->Init(&op_progress);
		// pkgApplyStatus(*cache->GetDepCache());
		handle_errors();
	}

	PkgDepCache(pkgDepCache* DepCache) : ptr(DepCache){};
};
