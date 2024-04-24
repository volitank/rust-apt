#pragma once
#include <apt-pkg/cachefile.h>
#include <apt-pkg/indexfile.h>
#include <apt-pkg/pkgcache.h>
#include <apt-pkg/pkgrecords.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/sourcelist.h>
#include <memory>
#include "rust/cxx.h"

#include "package.h"
#include "types.h"

/// Package Record Management:
struct PkgRecords {
	pkgRecords mutable records;
	pkgRecords::Parser mutable* parser;
	u_int64_t mutable last;

	inline bool already_has(u_int64_t index) const {
		if (last == index) { return true; }
		last = index;
		return false;
	}

	/// Moves the Records into the correct place.
	inline void ver_file_lookup(const VerFileIterator& file) const {
		if (this->already_has(file.Index())) { return; }
		this->parser = &records.Lookup(file);
	}

	/// Moves the Records into the correct place.
	inline void desc_file_lookup(const DescFileIterator& desc_file) const {
		if (this->already_has(desc_file.Index())) { return; }
		this->parser = &records.Lookup(desc_file);
	}

	/// Return the URI for a version as determined by it's package file.
	/// A version could have multiple package files and multiple URIs.
	inline rust::string ver_uri(const std::unique_ptr<IndexFile>& file) const {
		if (!parser) {
			throw std::runtime_error(
				"You have to run 'cache.ver_lookup()' or 'desc_lookup()' first!"
			);
		}
		return (*file)->ArchiveURI(parser->FileName());
	}

	/// Return the translated long description of a Package.
	inline rust::string long_desc() const { return handle_string(parser->LongDesc()); }

	/// Return the translated short description of a Package.
	inline rust::string short_desc() const { return handle_string(parser->ShortDesc()); }

	/// Return the Source package version string.
	inline rust::string get_field(rust::string field) const {
		return handle_string(parser->RecordField(field.c_str()));
	}

	/// Find the hash of a Version. Returns Result if there is no hash.
	inline rust::string hash_find(rust::string hash_type) const {
		auto hashes = parser->Hashes();
		auto hash = hashes.find(hash_type.c_str());
		if (hash == NULL) { throw std::runtime_error("Hash Not Found"); }
		return handle_string(hash->HashValue());
	}

	PkgRecords(pkgCacheFile* cache) : records(*cache->GetPkgCache()), parser(0), last(0){};

	/// UniquePtr Constructor
	static std::unique_ptr<PkgRecords> Unique(pkgCacheFile* cache) {
		return std::make_unique<PkgRecords>(cache);
	};
};
