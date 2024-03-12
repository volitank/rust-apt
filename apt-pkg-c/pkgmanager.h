#pragma once
#include <apt-pkg/acquire.h>
#include <apt-pkg/algorithms.h>
#include <apt-pkg/cachefile.h>
#include <apt-pkg/install-progress.h>
#include <apt-pkg/packagemanager.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/sourcelist.h>
#include <memory>

struct PackageManager {
	pkgPackageManager mutable* pkgmanager;

	inline void get_archives(
		const Cache& cache,
		const Records& records,
		DynAcquireProgress& callback
	) const {
		AcqTextStatus archive_progress(callback);
		pkgAcquire acquire(&archive_progress);

		// We probably need to let the user set their own pkgSourceList,
		// but there hasn't been a need to expose such in the Rust interface
		// yet. pkgSourceList sourcelist = *cache->GetSourceList();
		if (!pkgmanager->GetArchives(&acquire, cache.ptr->GetSourceList(), &records.records)) {
			handle_errors();
			throw std::runtime_error(
				"Internal Issue with rust-apt in pkgmanager_get_archives."
				" Please report this as an issue."
			);
		}

		pkgAcquire::RunResult result = acquire.Run(pulse_interval(callback));

		if (result != pkgAcquire::Continue) {
			// The other variants are either Failed or Cancelled
			// Failed will always have an error for us to handle
			// It's unsure if Cancelled would even require a bool
			// I believe this may be a Keyboard Interrupt situation
			handle_errors();
		}
	}

	inline void do_install(DynInstallProgress& callback) const {
		PackageManagerWrapper install_progress(callback);
		pkgPackageManager::OrderResult res = pkgmanager->DoInstall(&install_progress);

		if (res == pkgPackageManager::OrderResult::Completed) {
			return;
		} else if (res == pkgPackageManager::OrderResult::Failed) {
			handle_errors();
			throw std::runtime_error(
				"Internal Issue with rust-apt in pkgmanager_do_install."
				" DoInstall has failed but there was no error from apt."
				" Please report this as an issue."
			);
		} else if (res == pkgPackageManager::OrderResult::Incomplete) {
			// It's not clear that there would be any apt errors here,
			// But we'll try anyway. This is believed to be only for media swapping
			handle_errors();
			throw std::runtime_error(
				"Internal Issue with rust-apt in pkgmanager_do_install."
				" DoInstall returned Incomplete, media swaps are unsupported."
				" Please request media swapping as a feature."
			);
		} else {
			// If for whatever reason we manage to make it here (We shouldn't)
			// Attempt to handle any apt errors
			// And then fallback with a message to report with the result code.
			handle_errors();
			throw std::runtime_error(
				"Internal Issue with rust-apt in pkgmanager_do_install."
				" Please report this as an issue. OrderResult: " +
				res
			);
		}
	}

	PackageManager(pkgDepCache* depcache) : pkgmanager(_system->CreatePM(depcache)){};
};

struct ProblemResolver {
	pkgProblemResolver mutable resolver;

	/// Mark a package as protected, i.e. don't let its installation/removal state change when
	/// modifying packages during resolution.
	inline void protect(const Package& pkg) const { resolver.Protect(*pkg.ptr); }

	/// Try to resolve dependency problems by marking packages for installation and removal.
	inline void resolve(bool fix_broken, DynOperationProgress& callback) const {
		OpProgressWrapper op_progress(callback);
		resolver.Resolve(fix_broken, &op_progress);
		handle_errors();
	}

	ProblemResolver(pkgDepCache* depcache) : resolver(depcache){};
};

/// Create the problem resolver.
std::unique_ptr<ProblemResolver> create_problem_resolver(const Cache& cache) {
	return std::make_unique<ProblemResolver>(cache.ptr->GetDepCache());
}

std::unique_ptr<PackageManager> create_pkgmanager(const Cache& cache) {
	// Package Manager needs the DepCache initialized or else invalid memory reference.
	return std::make_unique<PackageManager>(cache.ptr->GetDepCache());
}
