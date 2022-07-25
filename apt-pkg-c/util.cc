#include <apt-pkg/configuration.h>
#include <apt-pkg/init.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/version.h>

#include "rust-apt/src/util.rs"

/// Compare two package version strings.
int32_t cmp_versions(rust::String ver1_rust, rust::String ver2_rust) {
	const char* ver1 = ver1_rust.c_str();
	const char* ver2 = ver2_rust.c_str();

	if (!_system) {
		pkgInitSystem(*_config, _system);
	}

	return _system->VS->DoCmpVersion(ver1, ver1 + strlen(ver1), ver2, ver2 + strlen(ver2));
}
