#pragma once
#include <apt-pkg/algorithms.h>

struct DynOperationProgress;

using PkgProblemResolver = pkgProblemResolver;

std::unique_ptr<PkgProblemResolver> problem_resolver_create(
const std::unique_ptr<PkgCacheFile>& cache);
void resolver_protect(
const std::unique_ptr<PkgProblemResolver>& resolver, const PackagePtr& pkg);
void resolver_resolve(const std::unique_ptr<PkgProblemResolver>& resolver,
bool fix_broken,
DynOperationProgress& op_progress);
