#include <apt-pkg/indexfile.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/policy.h>
#include <apt-pkg/sourcelist.h>

#include "rust-apt/src/package.rs"


/// Dependency types.
/// They must be duplicated here as getting them from apt would be translated.
const char* UntranslatedDepTypes[] = { "", "Depends", "PreDepends", "Suggests",
	"Recommends", "Conflicts", "Replaces", "Obsoletes", "Breaks", "Enhances" };


/// Wrap the PkgIterator into our PackagePtr Struct.
static PackagePtr wrap_package(pkgCache::PkgIterator pkg) {
	if (pkg.end()) {
		throw std::runtime_error("Package doesn't exist");
	}

	return PackagePtr{ std::make_unique<pkgCache::PkgIterator>(pkg) };
}


/// Wrap the VerIterator into our VersionPtr Struct.
static VersionPtr wrap_version(pkgCache::VerIterator ver) {
	if (ver.end()) {
		throw std::runtime_error("Version doesn't exist");
	}

	return VersionPtr{
		std::make_unique<pkgCache::VerIterator>(ver),
		std::make_unique<pkgCache::DescIterator>(ver.TranslatedDescription()),
	};
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

/// Return the version determined by a version string.
VersionPtr pkg_get_version(const PackagePtr& pkg, rust::string version_str) {
	auto ver_list = pkg.ptr->VersionList();
	for (; !ver_list.end(); ver_list++) {
		if (version_str == ver_list.VerStr()) {
			return wrap_version(ver_list);
		}
	}
	// This doesn't matter. We will be converting it into option on the rust side
	throw std::runtime_error("Version not found");
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


/// The list of packages that this package provides for.
rust::Vec<rust::string> ver_provides_list(const VersionPtr& ver) {
	rust::Vec<rust::string> list;

	for (pkgCache::PrvIterator pkg = ver.ptr->ProvidesList(); !pkg.end(); pkg++) {
		const char* name = pkg.Name();
		const char* version = pkg.ProvideVersion();

		if (version != NULL) {
			list.push_back(std::string(name) + std::string("/") + std::string(version));
		} else {
			list.push_back(std::string(name) + std::string("/"));
		}
	}

	return list;
}


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
