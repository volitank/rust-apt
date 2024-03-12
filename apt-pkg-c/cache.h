#pragma once
#include <apt-pkg/cachefile.h>
#include <apt-pkg/debfile.h>
#include <apt-pkg/error.h>
#include <apt-pkg/fileutl.h>
#include <apt-pkg/indexfile.h>
#include <apt-pkg/pkgcache.h>
#include <apt-pkg/policy.h>
#include <apt-pkg/sourcelist.h>
#include <apt-pkg/update.h>
#include "rust/cxx.h"

#include "rust-apt/src/raw/cache.rs"
#include "rust-apt/src/raw/progress.rs"

/// Update the package lists, handle errors and return a Result.
inline void Cache::update(DynAcquireProgress& callback) const {
	AcqTextStatus progress(callback);

	ListUpdate(progress, *ptr->GetSourceList(), pulse_interval(callback));
	handle_errors();
}

// Return a package by name.
inline Package Cache::unsafe_find_pkg(rust::string name) const noexcept {
	return Package{
		std::make_unique<PkgIterator>(safe_get_pkg_cache(ptr.get())->FindPkg(name.c_str()))};
}

inline Package Cache::begin() const {
	return Package{std::make_unique<PkgIterator>(safe_get_pkg_cache(ptr.get())->PkgBegin())};
}

/// The priority of the package as shown in `apt policy`.
inline int32_t Cache::priority(const Version& ver) const noexcept {
	return ptr->GetPolicy()->GetPriority(*ver.ptr);
}

inline DepCache Cache::create_depcache() const noexcept {
	return DepCache{std::make_unique<PkgDepCache>(ptr->GetDepCache())};
}

inline std::unique_ptr<Records> Cache::create_records() const noexcept {
	return Records::Unique(ptr);
}

inline void Cache::find_index(PackageFile& pkg_file) const noexcept {
	if (!pkg_file.index_file) {
		pkgIndexFile* index;

		if (!ptr->GetSourceList()->FindIndex(*pkg_file.ptr, index)) {
			_system->FindIndex(*pkg_file.ptr, index);
		}
		pkg_file.index_file = std::make_unique<IndexFile>(index);
	}
}

/// These should probably go under a index file binding;
/// Return true if the PackageFile is trusted.
inline bool Cache::is_trusted(PackageFile& pkg_file) const noexcept {
	this->find_index(pkg_file);
	return (*pkg_file.index_file)->IsTrusted();
}

/// Get the package list uris. This is the files that are updated with `apt update`.
inline rust::Vec<SourceURI> Cache::source_uris() const noexcept {
	pkgAcquire fetcher;
	rust::Vec<SourceURI> list;

	ptr->GetSourceList()->GetIndexes(&fetcher, true);
	pkgAcquire::UriIterator I = fetcher.UriBegin();
	for (; I != fetcher.UriEnd(); ++I) {
		list.push_back(SourceURI{I->URI, flNotDir(I->Owner->DestFile)});
	}
	return list;
}

inline Cache create_cache(rust::Slice<const rust::String> deb_files) {
	std::unique_ptr<pkgCacheFile> cache = std::make_unique<pkgCacheFile>();

	for (auto deb_str : deb_files) {
		std::string deb_string(deb_str.c_str());

		// Make sure this is a valid archive.
		// signal: 11, SIGSEGV: invalid memory reference
		FileFd fd(deb_string, FileFd::ReadOnly);
		debDebFile debfile(fd);
		handle_errors();

		// Add the deb to the cache.
		if (!cache->GetSourceList()->AddVolatileFile(deb_string)) {
			_error->Error("%s", ("Couldn't add '" + deb_string + "' to the cache.").c_str());
			handle_errors();
		}

		handle_errors();
	}

	return Cache{std::move(cache)};
}
