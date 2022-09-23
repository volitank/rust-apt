#include <apt-pkg/acquire.h>
#include <apt-pkg/install-progress.h>
#include <apt-pkg/packagemanager.h>
#include <apt-pkg/sourcelist.h>

#include "rust-apt/apt-pkg-c/util.h"
#include "rust-apt/src/cache.rs"
#include "rust-apt/src/pkgmanager.rs"
#include "rust-apt/src/progress.rs"

std::unique_ptr<PkgPackageManager> pkgmanager_create(
const std::unique_ptr<PkgCacheFile>& cache) {
	return std::unique_ptr<pkgPackageManager>(_system->CreatePM(*cache));
}

void pkgmanager_get_archives(const std::unique_ptr<PkgPackageManager>& pkgmanager,
const std::unique_ptr<PkgCacheFile>& cache,
Records& records,
DynAcquireProgress& callback) {
	AcqTextStatus archive_progress(callback);
	pkgAcquire acquire(&archive_progress);

	// We probably need to let the user set their own pkgSourceList,
	// but there hasn't been a need to expose such in the Rust interface yet.
	// pkgSourceList sourcelist = *cache->GetSourceList();
	if (!pkgmanager->GetArchives(
		&acquire, cache->GetSourceList(), &records.records->records)) {
		handle_errors();
		throw std::runtime_error(
		"Internal Issue with rust-apt in pkgmanager_get_archives."
		" Please report this as an issue.");
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

void pkgmanager_do_install(const std::unique_ptr<PkgPackageManager>& pkgmanager,
DynInstallProgress& callback) {
	PackageManagerWrapper install_progress(callback);
	pkgPackageManager::OrderResult res = pkgmanager->DoInstall(&install_progress);

	if (res == pkgPackageManager::OrderResult::Completed) {
		return;
	} else if (res == pkgPackageManager::OrderResult::Failed) {
		handle_errors();
		throw std::runtime_error(
		"Internal Issue with rust-apt in pkgmanager_do_install."
		" DoInstall has failed but there was no error from apt."
		" Please report this as an issue.");
	} else if (res == pkgPackageManager::OrderResult::Incomplete) {
		// It's not clear that there would be any apt errors here,
		// But we'll try anyway. This is believed to be only for media swapping
		handle_errors();
		throw std::runtime_error(
		"Internal Issue with rust-apt in pkgmanager_do_install."
		" DoInstall returned Incomplete, media swaps are unsupported."
		" Please request media swapping as a feature.");
	} else {
		// If for whatever reason we manage to make it here (We shouldn't)
		// Attempt to handle any apt errors
		// And then fallback with a message to report with the result code.
		handle_errors();
		throw std::runtime_error(
		"Internal Issue with rust-apt in pkgmanager_do_install."
		" Please report this as an issue. OrderResult: " +
		res);
	}
}
