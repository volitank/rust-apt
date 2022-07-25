#pragma once
#include "rust/cxx.h"
#include <cstdint>

/// Compare two package version strings.
int32_t cmp_versions(rust::String ver1_rust, rust::String ver2_rust);
