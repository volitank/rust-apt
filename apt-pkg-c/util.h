#pragma once
#include "rust/cxx.h"
#include <apt-pkg/algorithms.h>
#include <apt-pkg/install-progress.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/version.h>
#include <cstdint>

//#include "rust-apt/src/package.rs"

/// Internal Helper Functions.
/// Do not expose these on the Rust side - only for use on the C++ side.
///
/// Handle any apt errors and return result to rust.
inline void handle_errors() {
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

/// Handle the situation where a string is null and return a result to rust
inline const char* handle_str(const char* str) {
	if (!str || !strcmp(str, "")) {
		throw std::runtime_error("&str doesn't exist");
	}
	return str;
}

/// Check if a string exists and return a Result to rust
inline rust::string handle_string(std::string string) {
	if (string.empty()) {
		throw std::runtime_error("String doesn't exist");
	}
	return string;
}

//////////////////////////////////
/// End Internal Helper Functions.
//////////////////////////////////

/// Compare two package version strings.
inline int32_t cmp_versions(rust::String ver1_rust, rust::String ver2_rust) {
	const char* ver1 = ver1_rust.c_str();
	const char* ver2 = ver2_rust.c_str();

	if (!_system) {
		pkgInitSystem(*_config, _system);
	}

	return _system->VS->DoCmpVersion(ver1, ver1 + strlen(ver1), ver2, ver2 + strlen(ver2));
}

/// Return an APT-styled progress bar (`[####  ]`).
inline rust::String get_apt_progress_string(float percent, uint32_t output_width) {
	return APT::Progress::PackageManagerFancy::GetTextProgressStr(percent, output_width);
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
