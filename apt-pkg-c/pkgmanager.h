#pragma once
#include <apt-pkg/acquire.h>
#include <apt-pkg/algorithms.h>
#include <apt-pkg/cachefile.h>
#include <apt-pkg/install-progress.h>
#include <apt-pkg/packagemanager.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/sourcelist.h>
#include <memory>

#include "cache.h"
#include "rust-apt/src/progress.rs"

using OrderResult = pkgPackageManager::OrderResult;

struct PackageManager {
	pkgPackageManager mutable* pkgmanager;

	void get_archives(
		const PkgCacheFile& cache,
		const PkgRecords& records,
		AcqTextStatus& archive_progress
	) const {
		pkgAcquire acquire(&archive_progress);

		// We probably need to let the user set their own pkgSourcePkgCacheFileList,
		// but there hasn't been a need to expose such in the Rust interface
		// yet. pkgSourceList sourcelist = *cache->GetSourceList();

		if (!pkgmanager->GetArchives(
				&acquire, cache.unconst()->GetSourceList(), &records.records
			)) {
			handle_errors();
			throw std::runtime_error(
				"Internal Issue with rust-apt in pkgmanager_get_archives."
				" Please report this as an issue."
			);
		}

		pkgAcquire::RunResult result = acquire.Run(archive_progress.callback->pulse_interval());

		if (result != pkgAcquire::Continue) {
			// The other variants are either Failed or Cancelled
			// Failed will always have an error for us to handle
			// It's unsure if Cancelled would even require a bool
			// I believe this may be a Keyboard Interrupt situation
			handle_errors();
		}
	}

	OrderResult do_install_fd(i32 fd) const {
		APT::Progress::PackageManagerProgressFd install_progress(fd);
		return pkgmanager->DoInstall(&install_progress);
	}

	OrderResult do_install(InstallProgressFancy& callback) const {
		PackageManagerWrapper install_progress(callback);
		return pkgmanager->DoInstall(&install_progress);
	}

	PackageManager(pkgDepCache* depcache) : pkgmanager(_system->CreatePM(depcache)) {};
};

struct ProblemResolver {
	pkgProblemResolver mutable resolver;

	/// Mark a package as protected, i.e. don't let its installation/removal state change when
	/// modifying packages during resolution.
	void protect(const PkgIterator& pkg) const { resolver.Protect(pkg); }

	/// Try to resolve dependency problems by marking packages for installation and removal.
	void resolve(bool fix_broken, OperationProgress& callback) const {
		OpProgressWrapper op_progress(callback);
		resolver.Resolve(fix_broken, &op_progress);
		handle_errors();
	}

	ProblemResolver(pkgDepCache* depcache) : resolver(depcache) {};
};

/// Create the problem resolver.
UniquePtr<ProblemResolver> create_problem_resolver(const PkgDepCache& cache) {
	return std::make_unique<ProblemResolver>(cache.ptr);
}

UniquePtr<PackageManager> create_pkgmanager(const PkgDepCache& cache) {
	// Package Manager needs the DepCache initialized or else invalid memory reference.
	return std::make_unique<PackageManager>(cache.ptr);
}
