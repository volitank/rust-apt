#include <apt-pkg/acquire-item.h>
#include <apt-pkg/algorithms.h>
#include <apt-pkg/fileutl.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/policy.h>
#include <apt-pkg/sourcelist.h>
#include <apt-pkg/update.h>
#include <apt-pkg/version.h>

// Headers for the cxx bridge
#include "rust-apt/src/raw.rs"

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


/// Dependency types.
/// They must be duplicated here as getting them from apt would be translated.
const char* UntranslatedDepTypes[] = { "", "Depends", "PreDepends", "Suggests",
	"Recommends", "Conflicts", "Replaces", "Obsoletes", "Breaks", "Enhances" };


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


/// Create the Package Records.
Records pkg_records_create(const std::unique_ptr<PkgCacheFile>& cache) {
	return Records{
		std::make_unique<PkgRecords>(cache->GetPkgCache()),
	};
}


/// Create the depcache.
std::unique_ptr<PkgDepCache> depcache_create(const std::unique_ptr<PkgCacheFile>& cache) {
	pkgApplyStatus(*cache->GetDepCache());
	return std::make_unique<pkgDepCache>(*cache->GetDepCache());
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


/// Compare two package version strings.
int32_t cmp_versions(rust::String ver1_rust, rust::String ver2_rust) {
	const char* ver1 = ver1_rust.c_str();
	const char* ver2 = ver2_rust.c_str();

	if (!_system) {
		pkgInitSystem(*_config, _system);
	}

	return _system->VS->DoCmpVersion(ver1, ver1 + strlen(ver1), ver2, ver2 + strlen(ver2));
}

/// Package Functions:

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


/// Return the installed version of the package.
/// Ptr will be NULL if it's not installed.
VersionPtr pkg_current_version(const PackagePtr& pkg) {
	return wrap_version(pkg.ptr->CurrentVer());
}


/// Return the candidate version of the package.
/// Ptr will be NULL if there isn't a candidate.
VersionPtr pkg_candidate_version(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return wrap_version(cache->GetPolicy()->GetCandidateVer(*pkg.ptr));
}


/// Return a Vector of all the versions of a package.
rust::Vec<VersionPtr> pkg_version_list(const PackagePtr& pkg) {
	rust::Vec<VersionPtr> list;

	for (pkgCache::VerIterator I = pkg.ptr->VersionList(); !I.end(); I++) {
		list.push_back(wrap_version(I));
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


/// Check if the package is installed.
bool pkg_is_installed(const PackagePtr& pkg) { return pkg.ptr->CurrentVer(); }


/// Check if the package has versions.
/// If a package has no versions it is considered virtual.
bool pkg_has_versions(const PackagePtr& pkg) { return pkg.ptr->VersionList(); }


/// Check if a package provides anything.
/// Virtual packages may provide a real package.
/// This is how you would access the packages to satisfy it.
bool pkg_has_provides(const PackagePtr& pkg) { return pkg.ptr->ProvidesList(); }


/// Return true if the package is essential, otherwise false.
bool pkg_essential(const PackagePtr& pkg) {
	return ((*pkg.ptr)->Flags & pkgCache::Flag::Essential) != 0;
}


/// Get the fullname of a package.
/// More information on this in the package module.
rust::string get_fullname(const PackagePtr& pkg, bool pretty) {
	return pkg.ptr->FullName(pretty);
}


/// Get the name of a package.
rust::string pkg_name(const PackagePtr& pkg) { return pkg.ptr->Name(); }


/// Get the architecture of a package.
rust::string pkg_arch(const PackagePtr& pkg) { return pkg.ptr->Arch(); }


/// Get the ID of a package.
u_int32_t pkg_id(const PackagePtr& pkg) { return (*pkg.ptr)->ID; }


/// Get the current state of a package.
u_int8_t pkg_current_state(const PackagePtr& pkg) {
	return (*pkg.ptr)->CurrentState;
}


/// Get the installed state of a package.
u_int8_t pkg_inst_state(const PackagePtr& pkg) { return (*pkg.ptr)->InstState; }


/// Get the selected state of a package.
u_int8_t pkg_selected_state(const PackagePtr& pkg) {
	return (*pkg.ptr)->SelectedState;
}


/// Version Functions:

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


/// Return a Vector of all the dependencies of a version.
rust::Vec<DepContainer> dep_list(const VersionPtr& ver) {
	rust::Vec<DepContainer> depend_list;
	auto& cache = *ver.ptr->Cache();

	for (pkgCache::DepIterator dep = ver.ptr->DependsList(); !dep.end();) {
		DepContainer depend = DepContainer();
		pkgCache::DepIterator Start;
		pkgCache::DepIterator End;
		dep.GlobOr(Start, End);

		depend.dep_type = UntranslatedDepTypes[Start->Type];
		rust::Vec<BaseDep> list;

		while (true) {
			rust::string version;
			if (!Start->Version) {
				version = "";
			} else {
				version = Start.TargetVer();
			}

			list.push_back(BaseDep{
			Start.TargetPkg().Name(),
			version,
			Start.CompType(),
			UntranslatedDepTypes[Start->Type],
			std::make_shared<DepIterator>(cache, Start),
			});

			if (Start == End) {
				depend.dep_list = list;
				depend_list.push_back(depend);
				break;
			}

			Start++;
		}
	}
	return depend_list;
}


/// Return the parent package.
PackagePtr ver_parent(const VersionPtr& ver) {
	return wrap_package(ver.ptr->ParentPkg());
}


/// The architecture of a version.
rust::string ver_arch(const VersionPtr& ver) { return ver.ptr->Arch(); }


/// The version string of the version. "1.4.10"
rust::string ver_str(const VersionPtr& ver) { return ver.ptr->VerStr(); }


/// The section of the version as shown in `apt show`.
rust::string ver_section(const VersionPtr& ver) {
	// Some packages, such as msft teams, doesn't have a section.
	if (!ver.ptr->Section()) {
		return "None";
	}
	return ver.ptr->Section();
}


/// The priority string as shown in `apt show`.
rust::string ver_priority_str(const VersionPtr& ver) {
	return ver.ptr->PriorityType();
}


/// The name of the source package the version was built from.
rust::string ver_source_name(const VersionPtr& ver) {
	return ver.ptr->SourcePkgName();
}


/// The version of the source package.
rust::string ver_source_version(const VersionPtr& ver) {
	return ver.ptr->SourceVerStr();
}

/// The priority of the package as shown in `apt policy`.
int32_t ver_priority(const std::unique_ptr<PkgCacheFile>& cache, const VersionPtr& ver) {
	return cache->GetPolicy()->GetPriority(*ver.ptr);
}


/// The size of the .deb file.
u_int64_t ver_size(const VersionPtr& ver) { return (*ver.ptr)->Size; }


/// The uncompressed size of the .deb file.
u_int64_t ver_installed_size(const VersionPtr& ver) {
	return (*ver.ptr)->InstalledSize;
}


/// The ID of the version.
u_int32_t ver_id(const VersionPtr& ver) { return (*ver.ptr)->ID; }


/// If the version is able to be downloaded.
bool ver_downloadable(const VersionPtr& ver) { return ver.ptr->Downloadable(); }


/// Check if the version is currently installed.
bool ver_installed(const VersionPtr& ver) {
	return (*ver.ptr).ParentPkg().CurrentVer() == (*ver.ptr);
}


/// DepCache Information Accessors:

/// Is the Package upgradable?
///
/// `skip_depcache = true` increases performance by skipping the pkgDepCache
/// Skipping the depcache is very unnecessary if it's already been initialized
/// If you're not sure, set `skip_depcache = false`
bool pkg_is_upgradable(
const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg, bool skip_depcache) {
	if (!pkg.ptr->CurrentVer()) {
		return false;
	}
	if (skip_depcache) return is_upgradable(cache, *pkg.ptr);
	return (*cache->GetDepCache())[*pkg.ptr].Upgradable();
}


/// Is the Package auto installed? Packages marked as auto installed are usually depenencies.
bool pkg_is_auto_installed(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return is_auto_installed(cache, *pkg.ptr);
}


/// Is the Package able to be auto removed?
bool pkg_is_garbage(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].Garbage;
}


/// Is the Package marked for install?
bool pkg_marked_install(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].NewInstall();
}


/// Is the Package marked for upgrade?
bool pkg_marked_upgrade(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].Upgrade();
}


/// Is the Package marked for removal?
bool pkg_marked_delete(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].Delete();
}


/// Is the Package marked for keep?
bool pkg_marked_keep(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].Keep();
}


/// Is the Package marked for downgrade?
bool pkg_marked_downgrade(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].Downgrade();
}


/// Is the Package marked for reinstall?
bool pkg_marked_reinstall(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].ReInstall();
}


/// Is the installed Package broken?
bool pkg_is_now_broken(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].NowBroken();
}


/// Is the Package to be installed broken?
bool pkg_is_inst_broken(const std::unique_ptr<PkgCacheFile>& cache, const PackagePtr& pkg) {
	return (*cache->GetDepCache())[*pkg.ptr].InstBroken();
}


/// The number of packages marked for installation.
u_int32_t install_count(const std::unique_ptr<PkgCacheFile>& cache) {
	return cache->GetDepCache()->InstCount();
}


/// The number of packages marked for removal.
u_int32_t delete_count(const std::unique_ptr<PkgCacheFile>& cache) {
	return cache->GetDepCache()->DelCount();
}


/// The number of packages marked for keep.
u_int32_t keep_count(const std::unique_ptr<PkgCacheFile>& cache) {
	return cache->GetDepCache()->KeepCount();
}


/// The number of packages with broken dependencies in the cache.
u_int32_t broken_count(const std::unique_ptr<PkgCacheFile>& cache) {
	return cache->GetDepCache()->BrokenCount();
}


/// The size of all packages to be downloaded.
u_int64_t download_size(const std::unique_ptr<PkgCacheFile>& cache) {
	return cache->GetDepCache()->DebSize();
}


/// The amount of space required for installing/removing the packages,"
///
/// i.e. the Installed-Size of all packages marked for installation"
/// minus the Installed-Size of all packages for removal."
int64_t disk_size(const std::unique_ptr<PkgCacheFile>& cache) {
	return cache->GetDepCache()->UsrSize();
}


/// Package Record Management:

/// Moves the Records into the correct place.
void ver_file_lookup(Records& records, const PackageFile& pkg_file) {
	auto Index = pkg_file.ver_file->Index();
	if (records.records->last == Index) {
		return;
	}

	records.records->last = Index;
	records.records->parser = &records.records->records.Lookup(*pkg_file.ver_file);
}


/// Moves the Records into the correct place.
void desc_file_lookup(Records& records, const std::unique_ptr<DescIterator>& desc) {
	auto Index = desc->FileList().Index();
	if (records.records->last == Index) {
		return;
	}

	records.records->last = Index;
	records.records->parser = &records.records->records.Lookup(desc->FileList());
}


/// Return the URI for a version as determined by it's package file.
/// A version could have multiple package files and multiple URIs.
rust::string ver_uri(const Records& records,
const std::unique_ptr<PkgCacheFile>& cache,
const PackageFile& pkg_file) {
	pkgSourceList* SrcList = cache->GetSourceList();
	pkgIndexFile* Index;

	if (!SrcList->FindIndex(pkg_file.ver_file->File(), Index)) {
		_system->FindIndex(pkg_file.ver_file->File(), Index);
	}
	return Index->ArchiveURI(records.records->parser->FileName());
}


/// Return the translated long description of a Package.
rust::string long_desc(const Records& records) {
	return records.records->parser->LongDesc();
}


/// Return the translated short description of a Package.
rust::string short_desc(const Records& records) {
	return records.records->parser->ShortDesc();
}


/// Find the hash of a Version. Returns "KeyError" (lul python) if there is no hash.
rust::string hash_find(const Records& records, rust::string hash_type) {
	auto hashes = records.records->parser->Hashes();
	auto hash = hashes.find(hash_type.c_str());
	if (hash == NULL) {
		return "KeyError";
	}
	return hash->HashValue();
}


/// Dependency Functions:

/// Return a Vector of all versions that can satisfy a dependency.
rust::vec<VersionPtr> dep_all_targets(const BaseDep& dep) {
	rust::vec<VersionPtr> list;

	std::unique_ptr<pkgCache::Version*[]> versions(dep.ptr->AllTargets());
	for (pkgCache::Version** I = versions.get(); *I != 0; I++) {
		list.push_back(wrap_version(pkgCache::VerIterator(*dep.ptr->Cache(), *I)));
	}
	return list;
}
