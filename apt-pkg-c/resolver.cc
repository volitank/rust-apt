#include "rust-apt/src/resolver.rs"
#include "rust-apt/apt-pkg-c/util.h"
#include "rust-apt/src/cache.rs"
#include "rust-apt/src/progress.rs"


/// Create the problem resolver.
std::unique_ptr<PkgProblemResolver> problem_resolver_create(
const std::unique_ptr<PkgCacheFile>& cache) {
	return std::make_unique<PkgProblemResolver>(cache->GetDepCache());
}

/// Mark a package as protected, i.e. don't let its installation/removal state change when modifying packages during resolution.
void resolver_protect(
const std::unique_ptr<PkgProblemResolver>& resolver, const PackagePtr& pkg) {
	resolver->Protect(*pkg.ptr);
}

/// Try to resolve dependency problems by marking packages for installation and removal.
void resolver_resolve(const std::unique_ptr<PkgProblemResolver>& resolver,
bool fix_broken,
DynOperationProgress& callback) {
	OpProgressWrapper op_progress(callback);
	resolver->Resolve(fix_broken, &op_progress);
	handle_errors();
}
