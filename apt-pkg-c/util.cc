#include <apt-pkg/acquire.h>
#include <apt-pkg/algorithms.h>
#include <apt-pkg/configuration.h>
#include <apt-pkg/init.h>
#include <apt-pkg/install-progress.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/policy.h>
#include <apt-pkg/version.h>

#include "rust-apt/src/util.rs"

/// Handle any apt errors and return result to rust.
void handle_errors() {
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

/// Return a result to rust in the event the string is empty.
rust::string handle_null_str(const char* str) {
	if (!str) {
		throw std::runtime_error("Unknown");
	}
	return str;
}

/// Wrap the PkgIterator into our PackagePtr Struct.
PackagePtr wrap_package(pkgCache::PkgIterator pkg) {
	if (pkg.end()) {
		throw std::runtime_error("Package doesn't exist");
	}

	return PackagePtr{ std::make_unique<pkgCache::PkgIterator>(pkg) };
}

/// Wrap the VerIterator into our VersionPtr Struct.
VersionPtr wrap_version(pkgCache::VerIterator ver) {
	if (ver.end()) {
		throw std::runtime_error("Version doesn't exist");
	}

	return VersionPtr{
		std::make_unique<pkgCache::VerIterator>(ver),
		std::make_unique<pkgCache::DescIterator>(ver.TranslatedDescription()),
	};
}

/// Wrap PkgFileIterator into PackageFile Struct.
PackageFile wrap_pkg_file(pkgCache::PkgFileIterator pkg_file) {
	return PackageFile{
		std::make_unique<PkgFile>(pkg_file),
	};
}

/// Wrap VerFileIterator into VersionFile Struct.
VersionFile wrap_ver_file(pkgCache::VerFileIterator ver_file) {
	return VersionFile{
		std::make_unique<pkgCache::VerFileIterator>(ver_file),
	};
}

/// Determine if the package is upgradable without the depcache.
bool is_upgradable(
const std::unique_ptr<PkgCacheFile>& cache, const pkgCache::PkgIterator& pkg) {
	pkgCache::VerIterator inst = pkg.CurrentVer();
	if (!inst) return false;

	pkgCache::VerIterator cand = cache->GetPolicy()->GetCandidateVer(pkg);
	if (!cand) return false;

	return inst != cand;
}

/// Determine if the package is auto removable.
bool is_auto_removable(
const std::unique_ptr<PkgCacheFile>& cache, const pkgCache::PkgIterator& pkg) {
	pkgDepCache::StateCache state = (*cache->GetDepCache())[pkg];
	return ((pkg.CurrentVer() || state.NewInstall()) && state.Garbage);
}

/// Determine if the package is auto installed.
bool is_auto_installed(
const std::unique_ptr<PkgCacheFile>& cache, const pkgCache::PkgIterator& pkg) {
	pkgDepCache::StateCache state = (*cache->GetDepCache())[pkg];
	return state.Flags & pkgCache::Flag::Auto;
}

/// Compare two package version strings.
int32_t cmp_versions(rust::String ver1_rust, rust::String ver2_rust) {
	const char* ver1 = ver1_rust.c_str();
	const char* ver2 = ver2_rust.c_str();

	if (!_system) {
		pkgInitSystem(*_config, _system);
	}

	return _system->VS->DoCmpVersion(ver1, ver1 + strlen(ver1), ver2, ver2 + strlen(ver2));
}

/// Return an APT-styled progress bar (`[####  ]`).
rust::String get_apt_progress_string(float percent, uint32_t output_width) {
	return APT::Progress::PackageManagerFancy::GetTextProgressStr(percent, output_width);
}

/// Lock the APT lockfile.
void apt_lock() {
	_system->Lock();
	handle_errors();
}

/// Unlock the APT lockfile.
void apt_unlock() {
	// This can only throw an error that says "Not Locked"
	// By setting NoErrors true, this will return false instead
	// This is largely irrelevant and will be a void function
	_system->UnLock(true);
}

/// Lock the Dpkg lockfile.
void apt_lock_inner() {
	_system->LockInner();
	handle_errors();
}

/// Unlock the Dpkg lockfile.
void apt_unlock_inner() {
	// UnlockInner can not throw an error and always returns true.
	_system->UnLockInner();
}

/// Check if the lockfile is locked.
bool apt_is_locked() { return _system->IsLocked(); }
