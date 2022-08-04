#pragma once
#include "rust/cxx.h"
#include <apt-pkg/cachefile.h>
#include <apt-pkg/indexfile.h>
#include <apt-pkg/pkgcache.h>
#include <apt-pkg/pkgrecords.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/sourcelist.h>
#include <memory>

struct PkgRecords {

	pkgRecords records;
	// Parser doesn't want to work as a UniquePtr
	pkgRecords::Parser* parser;

	u_int32_t last;

	PkgRecords(pkgCache* cache) : records(*cache), last(0){};
};

// Rust Shared Structs
struct Records;

/// Package Record Management:

Records records_create(const std::unique_ptr<PkgCacheFile>& cache);

void ver_file_lookup(Records& records, const VersionFile& pkg_file);

void desc_file_lookup(Records& records, const std::unique_ptr<DescIterator>& desc);

rust::string ver_uri(const Records& records,
const std::unique_ptr<PkgCacheFile>& cache,
const VersionFile& ver_file);

rust::string long_desc(const Records& records);

rust::string short_desc(const Records& records);

rust::string hash_find(const Records& records, rust::string hash_type);
