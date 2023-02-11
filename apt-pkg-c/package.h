#pragma once
#include "rust/cxx.h"
#include <apt-pkg/cachefile.h>
#include <apt-pkg/policy.h>
#include <memory>

#include "rust-apt/src/raw/package.rs"
#include "util.h"

inline rust::Str Provider::name() const noexcept { return ptr->Name(); }

inline rust::Str Provider::version_str() const {
	return handle_str(ptr->ProvideVersion());
}

inline void Provider::raw_next() const noexcept { ++(*ptr); }
inline bool Provider::end() const noexcept { return ptr->end(); }

inline Package Provider::target_pkg() const noexcept {
	return Package{ std::make_unique<PkgIterator>(ptr->OwnerPkg()) };
}

inline Version Provider::target_ver() const noexcept {
	return Version{ std::make_unique<VerIterator>(ptr->OwnerVer()) };
}

inline Provider Provider::unique() const noexcept {
	return Provider{ std::make_unique<PrvIterator>(*ptr) };
}


/// The path to the PackageFile
inline rust::Str PackageFile::filename() const {
	return handle_str(ptr->FileName());
}

/// The Archive of the PackageFile. ex: unstable
inline rust::Str PackageFile::archive() const {
	return handle_str(ptr->Archive());
}

/// The Origin of the PackageFile. ex: Debian
inline rust::Str PackageFile::origin() const {
	return handle_str(ptr->Origin());
}

/// The Codename of the PackageFile. ex: main, non-free
inline rust::Str PackageFile::codename() const {
	return handle_str(ptr->Codename());
}

/// The Label of the PackageFile. ex: Debian
inline rust::Str PackageFile::label() const { return handle_str(ptr->Label()); }

/// The Hostname of the PackageFile. ex: deb.debian.org
inline rust::Str PackageFile::site() const { return handle_str(ptr->Site()); }

/// The Component of the PackageFile. ex: sid
inline rust::Str PackageFile::component() const {
	return handle_str(ptr->Component());
}

/// The Architecture of the PackageFile. ex: amd64
inline rust::Str PackageFile::arch() const {
	return handle_str(ptr->Architecture());
}

/// The Index Type of the PackageFile. Known values are:
///
/// Debian Package Index, Debian Translation Index, Debian dpkg status file,
inline rust::Str PackageFile::index_type() const {
	return handle_str(ptr->IndexType());
}

/// The Index number of the PackageFile
inline uint64_t PackageFile::index() const noexcept { return ptr->Index(); }


// Return the package file object.
inline PackageFile DescriptionFile::pkg_file() const noexcept {
	return PackageFile{ std::make_unique<PkgFileIterator>(ptr->File()), NULL };
}

// Return the Index of the Package File.
inline uint64_t DescriptionFile::index() const noexcept { return ptr->Index(); }

// Increment the iterator one
inline void DescriptionFile::raw_next() const noexcept { ++(*ptr); }

// Checks if the pointer is null meaning there is no more.
inline bool DescriptionFile::end() const noexcept { return ptr->end(); }

inline DescriptionFile DescriptionFile::unique() const noexcept {
	return DescriptionFile{ std::make_unique<DescFileIterator>(*ptr) };
}

// Return the VersionFile object.
inline PackageFile VersionFile::pkg_file() const noexcept {
	return PackageFile{ std::make_unique<PkgFileIterator>(ptr->File()), NULL };
}

// Return the Index of the VersionFile.
inline uint64_t VersionFile::index() const noexcept { return ptr->Index(); }

// Increment the iterator one
inline void VersionFile::raw_next() const noexcept { ++(*ptr); }

// Checks if the pointer is null meaning there is no more.
inline bool VersionFile::end() const noexcept { return ptr->end(); }

inline VersionFile VersionFile::unique() const noexcept {
	return VersionFile{ std::make_unique<VerFileIterator>(*ptr) };
}


/// String representation of the dependency compare type
/// "","<=",">=","<",">","=","!="
inline rust::Str Dependency::comp_type() const {
	return handle_str(ptr->CompType());
}

inline uint32_t Dependency::index() const noexcept { return ptr->Index(); }

/// u8 representation of the DepType. Will be converted to Enum in rust
inline uint8_t Dependency::dep_type() const noexcept { return (*ptr)->Type; }

// Return true if this dep is Or'd with the next. The last dep in the or group will return False.
inline bool Dependency::compare_op() const noexcept {
	return ((*ptr)->CompareOp & pkgCache::Dep::Or) == pkgCache::Dep::Or;
}

inline rust::Str Dependency::target_ver() const {
	return handle_str(ptr->TargetVer());
}

inline Package Dependency::target_pkg() const noexcept {
	return Package{ std::make_unique<PkgIterator>(ptr->TargetPkg()) };
}

// This should be tested. I'm not entirely sure this is even going to work.
inline Version Dependency::all_targets() const noexcept {
	return Version{ std::make_unique<VerIterator>(*ptr->Cache(), *ptr->AllTargets()) };
}

/// Increment the Dep Iterator once
inline void Dependency::raw_next() const noexcept { ++(*ptr); }
/// Is the pointer null, basically
inline bool Dependency::end() const noexcept { return ptr->end(); }

inline Dependency Dependency::unique() const noexcept {
	return Dependency{ std::make_unique<DepIterator>(*ptr) };
}

/// The ID of the version.
inline uint32_t Version::id() const noexcept { return (*ptr)->ID; }

/// The version string of the version. "1.4.10"
inline rust::Str Version::version() const noexcept { return ptr->VerStr(); }

/// The architecture of a version.
inline rust::Str Version::arch() const noexcept { return ptr->Arch(); }

/// The section of the version as shown in `apt show`.
inline rust::Str Version::section() const {
	// Some packages, such as msft teams, doesn't have a section.
	return handle_str(ptr->Section());
}

/// The priority string as shown in `apt show`.
inline rust::Str Version::priority_str() const {
	return handle_str(ptr->PriorityType());
}

/// The size of the .deb file.
inline uint64_t Version::size() const noexcept { return (*ptr)->Size; }

/// The uncompressed size of the .deb file.
inline uint64_t Version::installed_size() const noexcept {
	return (*ptr)->InstalledSize;
}

/// True if the version is able to be downloaded.
inline bool Version::is_downloadable() const noexcept {
	return ptr->Downloadable();
}

/// True if the version is currently installed.
inline bool Version::is_installed() const noexcept {
	return ptr->ParentPkg().CurrentVer() == *ptr;
}

// This is for backend records lookups. You can also get package files from here.
inline DescriptionFile Version::unsafe_description_file() const noexcept {
	return DescriptionFile{ std::make_unique<DescFileIterator>(
	ptr->TranslatedDescription().FileList()) };
}

// You go through here to get the package files.
inline VersionFile Version::unsafe_version_file() const noexcept {
	return VersionFile{ std::make_unique<VerFileIterator>(ptr->FileList()) };
}

// 	/// Return the parent package. TODO: This probably isn't going to work rn
// 	inline pkgCache::PkgIterator parent() const noexcept { return ptr->ParentPkg(); }

/// Always contains the name, even if it is the same as the binary name
inline rust::Str Version::source_name() const noexcept {
	return ptr->SourcePkgName();
}

// Always contains the version string, even if it is the same as the binary version
inline rust::Str Version::source_version() const noexcept {
	return ptr->SourceVerStr();
}

inline Dependency Version::unsafe_depends() const noexcept {
	return Dependency{ std::make_unique<DepIterator>(ptr->DependsList()) };
}


inline Provider Version::unsafe_provides() const noexcept {
	return Provider{ std::make_unique<PrvIterator>(ptr->ProvidesList()) };
}

inline void Version::raw_next() const noexcept { ++(*ptr); }
inline bool Version::end() const noexcept { return ptr->end(); }
inline Version Version::unique() const noexcept {
	return Version{ std::make_unique<VerIterator>(*ptr) };
}


inline rust::Str Package::name() const noexcept { return ptr->Name(); }
inline rust::Str Package::arch() const noexcept { return ptr->Arch(); }
inline rust::String Package::fullname(bool Pretty) const noexcept {
	return ptr->FullName(Pretty);
}

inline u_int32_t Package::id() const noexcept { return (*ptr)->ID; }
inline u_int8_t Package::current_state() const noexcept {
	return (*ptr)->CurrentState;
}
inline u_int8_t Package::inst_state() const noexcept {
	return (*ptr)->InstState;
}
inline u_int8_t Package::selected_state() const noexcept {
	return (*ptr)->SelectedState;
}

/// Return the installed version of the package.
/// Ptr will be NULL if it's not installed.
inline Version Package::unsafe_current_version() const noexcept {
	return Version{ std::make_unique<VerIterator>(ptr->CurrentVer()) };
}

/// True if the package is essential.
inline bool Package::is_essential() const noexcept {
	return ((*ptr)->Flags & pkgCache::Flag::Essential) != 0;
}

inline Version Package::unsafe_version_list() const noexcept {
	return Version{ std::make_unique<VerIterator>(ptr->VersionList()) };
}

inline Provider Package::unsafe_provides() const noexcept {
	return Provider{ std::make_unique<PrvIterator>(ptr->ProvidesList()) };
}

inline void Package::raw_next() const noexcept { ++(*ptr); }
inline bool Package::end() const noexcept { return this->ptr->end(); }
inline Package Package::unique() const noexcept {
	return Package{ std::make_unique<PkgIterator>(*ptr) };
}
