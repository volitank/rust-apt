#include <apt-pkg/acquire-item.h>
#include <apt-pkg/algorithms.h>
#include <apt-pkg/fileutl.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/policy.h>
#include <apt-pkg/sourcelist.h>
#include <apt-pkg/update.h>
#include <apt-pkg/version.h>

// Headers for the cxx bridge
#include "rust-apt/src/cache.rs"
#include "rust-apt/src/progress.rs"

/// Helper Functions:

/// Handle any apt errors and return result to rust.
static void handle_errors() {
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


/// Wrap the PkgIterator into our PackagePtr Struct.
static PackagePtr wrap_package(pkgCache::PkgIterator pkg) {
	if (pkg.end()) {
		return PackagePtr{ NULL };
	}

	return PackagePtr{ std::make_unique<pkgCache::PkgIterator>(pkg) };
}


/// Wrap the VerIterator into our VersionPtr Struct.
static VersionPtr wrap_version(pkgCache::VerIterator ver) {
	if (ver.end()) {
		return VersionPtr{ NULL, NULL };
	}

	return VersionPtr{
		std::make_unique<pkgCache::VerIterator>(ver),
		std::make_unique<pkgCache::DescIterator>(ver.TranslatedDescription()),
	};
}


static bool is_upgradable(
const std::unique_ptr<PkgCacheFile>& cache, const pkgCache::PkgIterator& pkg) {
	pkgCache::VerIterator inst = pkg.CurrentVer();
	if (!inst) return false;

	pkgCache::VerIterator cand = cache->GetPolicy()->GetCandidateVer(pkg);
	if (!cand) return false;

	return inst != cand;
}


static bool is_auto_removable(
const std::unique_ptr<PkgCacheFile>& cache, const pkgCache::PkgIterator& pkg) {
	pkgDepCache::StateCache state = (*cache->GetDepCache())[pkg];
	return ((pkg.CurrentVer() || state.NewInstall()) && state.Garbage);
}


static bool is_auto_installed(
const std::unique_ptr<PkgCacheFile>& cache, const pkgCache::PkgIterator& pkg) {
	pkgDepCache::StateCache state = (*cache->GetDepCache())[pkg];
	return state.Flags & pkgCache::Flag::Auto;
}


/// Main Initializers for apt:

/// Create the CacheFile.
std::unique_ptr<PkgCacheFile> pkg_cache_create() {
	return std::make_unique<PkgCacheFile>();
}


/// Update the package lists, handle errors and return a Result.
void cache_update(const std::unique_ptr<PkgCacheFile>& cache, DynUpdateProgress& callback) {
	AcqTextStatus progress(callback);

	ListUpdate(progress, *cache->GetSourceList(), pulse_interval(callback));
	handle_errors();
}


/// Get the package list uris. This is the files that are updated with `apt update`.
rust::Vec<SourceFile> source_uris(const std::unique_ptr<PkgCacheFile>& cache) {
	pkgAcquire fetcher;
	rust::Vec<SourceFile> list;

	cache->GetSourceList()->GetIndexes(&fetcher, true);
	pkgAcquire::UriIterator I = fetcher.UriBegin();

	for (; I != fetcher.UriEnd(); ++I) {
		list.push_back(SourceFile{ I->URI, flNotDir(I->Owner->DestFile) });
	}
	return list;
}

/// Returns a Vector of all the packages in the cache.
rust::Vec<PackagePtr> pkg_list(
const std::unique_ptr<PkgCacheFile>& cache, const PackageSort& sort) {
	rust::vec<PackagePtr> list;
	pkgCache::PkgIterator pkg;

	for (pkg = cache->GetPkgCache()->PkgBegin(); !pkg.end(); pkg++) {

		if ((sort.virtual_pkgs != Sort::Enable) &&
		((sort.virtual_pkgs == Sort::Disable && !pkg.VersionList()) ||
		(sort.virtual_pkgs == Sort::Reverse && pkg.VersionList()))) {
			continue;
		}

		if ((sort.upgradable != Sort::Disable) &&
		((sort.upgradable == Sort::Enable && !is_upgradable(cache, pkg)) ||
		(sort.upgradable == Sort::Reverse && is_upgradable(cache, pkg)))) {
			continue;
		}

		if ((sort.installed != Sort::Disable) &&
		((sort.installed == Sort::Enable && !pkg.CurrentVer()) ||
		(sort.installed == Sort::Reverse && pkg.CurrentVer()))) {
			continue;
		}

		if ((sort.auto_installed != Sort::Disable) &&
		((sort.auto_installed == Sort::Enable && !is_auto_installed(cache, pkg)) ||
		(sort.auto_installed == Sort::Reverse && is_auto_installed(cache, pkg)))) {
			continue;
		}

		if ((sort.auto_removable != Sort::Disable) &&
		((sort.auto_removable == Sort::Enable && !is_auto_removable(cache, pkg)) ||
		(sort.auto_removable == Sort::Reverse && is_auto_removable(cache, pkg)))) {
			continue;
		}

		list.push_back(wrap_package(pkg));
	}
	return list;
}


/// Return a Vector of all the package files for a version.
rust::vec<PackageFile> pkg_file_list(
const std::unique_ptr<PkgCacheFile>& cache, const VersionPtr& ver) {
	rust::vec<PackageFile> list;
	pkgCache::VerFileIterator v_file = ver.ptr->FileList();

	for (; !v_file.end(); v_file++) {
		pkgSourceList* SrcList = cache->GetSourceList();
		pkgIndexFile* Index;
		if (!SrcList->FindIndex(v_file.File(), Index)) {
			_system->FindIndex(v_file.File(), Index);
		}
		list.push_back(PackageFile{
		std::make_unique<pkgCache::VerFileIterator>(v_file),
		std::make_unique<pkgCache::PkgFileIterator>(v_file.File()),
		});
	}
	return list;
}


/// Return a Vector of all the versions of a package.
rust::Vec<VersionPtr> pkg_version_list(const PackagePtr& pkg) {
	rust::Vec<VersionPtr> list;

	for (pkgCache::VerIterator I = pkg.ptr->VersionList(); !I.end(); I++) {
		list.push_back(wrap_version(I));
	}
	return list;
}


/// Return a Vector of all the packages that provide another. steam:i386 provides steam.
rust::Vec<PackagePtr> pkg_provides_list(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool cand_only) {
	pkgCache::PrvIterator provide = pkg.ptr->ProvidesList();
	std::set<std::string> set;
	rust::vec<PackagePtr> list;

	for (; !provide.end(); provide++) {
		pkgCache::PkgIterator pkg = provide.OwnerPkg();
		bool is_cand = (provide.OwnerVer() == cache->GetPolicy()->GetCandidateVer(pkg));
		// If cand_only is true, then we check if ithe package is candidate.
		if (!cand_only || is_cand) {
			// Make sure we do not have duplicate packags.
			if (!set.insert(pkg.FullName()).second) {
				continue;
			}

			list.push_back(wrap_package(pkg));
		}
	}
	return list;
}


/// Return a package by name. Ptr will be NULL if the package doesn't exist.
PackagePtr pkg_cache_find_name(const std::unique_ptr<PkgCacheFile>& cache, rust::string name) {
	return wrap_package(cache->GetPkgCache()->FindPkg(name.c_str()));
}


/// Return a package by name and architecture.
/// Ptr will be NULL if the package doesn't exist.
PackagePtr pkg_cache_find_name_arch(
const std::unique_ptr<PkgCacheFile>& cache, rust::string name, rust::string arch) {
	return wrap_package(cache->GetPkgCache()->FindPkg(name.c_str(), arch.c_str()));
}
