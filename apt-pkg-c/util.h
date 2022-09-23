#pragma once
#include "rust/cxx.h"
#include <cstdint>

/// Handle any apt errors and return result to rust.
/// Do not expose this on the Rust side - this is just for use on the C++ side.
void handle_errors();

/// Compare two package version strings.
int32_t cmp_versions(rust::String ver1_rust, rust::String ver2_rust);

/// Return an APT-styled progress bar (`[####  ]`).
rust::String get_apt_progress_string(float percent, uint32_t output_width);

/// Lock the APT lockfile.
void apt_lock();

/// Unock the APT lockfile.
void apt_unlock();

/// Lock the Dpkg lockfile.
void apt_lock_inner();

/// Unlock the Dpkg lockfile.
void apt_unlock_inner();

/// Check if the lockfile is locked.
bool apt_is_locked();
