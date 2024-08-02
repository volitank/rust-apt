#pragma once
#include <apt-pkg/algorithms.h>
#include <apt-pkg/cachefile.h>
#include <apt-pkg/install-progress.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/version.h>
#include <cstdint>
#include <sstream>
#include "rust/cxx.h"

#include "types.h"

/// Internal Helper Functions.
/// Do not expose these on the Rust side - only for use on the C++ side.
///
/// Handle any apt errors and return result to rust.
inline void handle_errors() {
	// !_error->empty() will cause a Result when there are only warnings
	// Instead use PendingErr()
	// Actual handling of the errors is done in rust
	if (_error->PendingError()) { throw std::runtime_error("convert to AptErrors"); }
}

/// Handle the situation where a string is null and return a result to rust
inline const char* handle_str(const char* str) {
	if (!str || !strcmp(str, "")) { throw std::runtime_error("&str doesn't exist"); }
	return str;
}

/// Check if a string exists and return a Result to rust
inline String handle_string(std::string string) {
	if (string.empty()) { throw std::runtime_error("String doesn't exist"); }
	return string;
}

//////////////////////////////////
/// End Internal Helper Functions.
//////////////////////////////////

/// Compare two package version strings.
inline i32 cmp_versions(str ver1, str ver2) {
	if (!_system) { pkgInitSystem(*_config, _system); }

	const char* end1 = ver1.begin() + strlen(ver1.begin());
	const char* end2 = ver2.begin() + strlen(ver2.begin());

	return _system->VS->DoCmpVersion(ver1.begin(), end1, ver2.begin(), end2);
}

/// Return an APT-styled progress bar (`[####  ]`).
inline String get_apt_progress_string(f32 percent, u32 output_width) {
	return APT::Progress::PackageManagerFancy::GetTextProgressStr(percent, output_width);
}

inline String quote_string(str string, String bad) {
	return QuoteString(std::string(string), bad.c_str());
}

/// Lock the APT lockfile.
inline void apt_lock() {
	_system->Lock();
	handle_errors();
}

/// Unlock the APT lockfile.
inline void apt_unlock() {
	// This can only throw an error that says "Not Locked"
	// By setting NoErrors true, this will return false instead
	// This is largely irrelevant and will be a void function
	_system->UnLock(true);
}

/// Lock the Dpkg lockfile.
inline void apt_lock_inner() {
	_system->LockInner();
	handle_errors();
}

/// Unlock the Dpkg lockfile.
inline void apt_unlock_inner() {
	// UnlockInner can not throw an error and always returns true.
	_system->UnLockInner();
}

/// Check if the lockfile is locked.
inline bool apt_is_locked() { return _system->IsLocked(); }
