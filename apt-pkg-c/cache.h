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

// Defines the callbacks code that's generated for progress
#include "rust-apt/src/acquire.rs"

#include "depcache.h"
#include "records.h"
#include "types.h"

struct PkgCacheFile : public pkgCacheFile {
	// Maybe we use this if we don't want pin_mut() all over the place in Rust.
	PkgCacheFile* unconst() const { return const_cast<PkgCacheFile*>(this); }

	/// Update the package lists, handle errors and return a Result.
	void update(AcqTextStatus& progress) const {
		ListUpdate(
			progress, *this->unconst()->GetSourceList(), progress.callback->pulse_interval()
		);
		handle_errors();
	}

	// Return a package by name.
	UniquePtr<PkgIterator> find_pkg(str name) const {
		return std::make_unique<PkgIterator>(
			this->unconst()->GetPkgCache()->FindPkg(APT::StringView(name.begin(), name.length()))
		);
	}

	UniquePtr<PkgIterator> begin() const {
		return std::make_unique<PkgIterator>(this->unconst()->GetPkgCache()->PkgBegin());
	}

	/// The priority of the package as shown in `apt policy`.
	int32_t priority(const VerIterator& ver) const {
		return this->unconst()->GetPolicy()->GetPriority(ver);
	}

	UniquePtr<PkgDepCache> create_depcache() const {
		return std::make_unique<PkgDepCache>(this->unconst()->GetDepCache());
	}

	UniquePtr<PkgRecords> create_records() const {
		return std::make_unique<PkgRecords>(this->unconst());
	}

	UniquePtr<SourceRecords> source_records() const {
		auto records = std::make_unique<SourceRecords>(this->unconst()->GetSourceList());
		handle_errors();
		return records;
	}

	UniquePtr<IndexFile> find_index(const PkgFileIterator& file) const {
		pkgIndexFile* index;
		if (!this->unconst()->GetSourceList()->FindIndex(file, index)) {
			_system->FindIndex(file, index);
		}
		return std::make_unique<IndexFile>(index);
	}

	bool get_indexes(const PkgAcquire& fetcher) const {
		return this->unconst()->GetSourceList()->GetIndexes(fetcher.ptr, true);
	}

	PkgCacheFile() : pkgCacheFile() {};
};

inline UniquePtr<PkgCacheFile> create_cache(rust::Slice<const str> volatile_files) {
	UniquePtr<PkgCacheFile> cache = std::make_unique<PkgCacheFile>();

	for (auto file_str : volatile_files) {
		std::string file_string(file_str);
		// Add the file to the cache.
		if (!cache->GetSourceList()->AddVolatileFile(file_string)) {
			_error->Error("%s", ("Couldn't add '" + file_string + "' to the cache.").c_str());
		}
	}

	// Building the pkg caches can cause an error that might not
	// Get propagated until you get a pkg which shouldn't have errors.
	// See https://gitlab.com/volian/rust-apt/-/issues/24
	cache->GetPkgCache();
	handle_errors();

	return cache;
}
