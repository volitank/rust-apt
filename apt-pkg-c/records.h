#pragma once
#include <apt-pkg/cachefile.h>
#include <apt-pkg/indexfile.h>
#include <apt-pkg/pkgcache.h>
#include <apt-pkg/pkgrecords.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/sourcelist.h>
#include <apt-pkg/srcrecords.h>
#include <memory>
#include "rust/cxx.h"

#include "package.h"
#include "types.h"

struct IndexFile {
	pkgIndexFile* ptr;

	String archive_uri(str filename) const { return ptr->ArchiveURI(std::string(filename)); }
	bool is_trusted() const { return ptr->IsTrusted(); }

	IndexFile(pkgIndexFile* file) : ptr(file){};
};

struct Parser {
	pkgRecords::Parser& ptr;

	String short_desc() const { return handle_string(ptr.ShortDesc()); }
	String long_desc() const { return handle_string(ptr.LongDesc()); }
	String filename() const { return ptr.FileName(); }

	// TODO: Maybe look into this more if there is time. I was trying to save an allocation
	// ptr.RecordField(field.begin())
	// This will work with String.as_str()
	// cache.records().get_field(&"Maintainer".to_string())
	// This will not work with just "Maintainer" string literal

	/// Return the Source package version String.
	String get_field(String field) const { return handle_string(ptr.RecordField(field.c_str())); }

	// TODO: Lets Go Ahead and Bind HashStrings while we're here ffs
	/// Find the hash of a Version. Returns Result if there is no hash.
	String hash_find(String hash_type) const {
		auto hashes = ptr.Hashes();
		auto hash = hashes.find(hash_type.c_str());
		if (hash == NULL) { throw std::runtime_error("Hash Not Found"); }
		return handle_string(hash->HashValue());
	}

	Parser(pkgRecords::Parser& parser) : ptr(parser){};
};

struct PkgRecords {
	pkgRecords mutable records;

	UniquePtr<Parser> ver_lookup(const VerFileIterator& file) const {
		return std::make_unique<Parser>(records.Lookup(file));
	}

	/// Moves the Records into the correct place.
	UniquePtr<Parser> desc_lookup(const DescIterator& desc) const {
		return std::make_unique<Parser>(records.Lookup(desc.FileList()));
	}

	PkgRecords(pkgCacheFile* cache) : records(*cache->GetPkgCache()){};
};

struct SourceParser {
	pkgSrcRecords::Parser* ptr;

	String as_str() const { return ptr->AsStr(); }
	String package() const { return ptr->Package(); }
	String version() const { return ptr->Version(); }
	String maintainer() const { return ptr->Maintainer(); }
	String section() const { return ptr->Section(); }
	bool end() const { return ptr == 0; }

	SourceParser(pkgSrcRecords::Parser* parser) : ptr(parser){};
};

struct SourceRecords {
	pkgSrcRecords mutable records;

	void restart() const { records.Restart(); }
	UniquePtr<SourceParser> find(String name, bool src_only) const {
		return std::make_unique<SourceParser>(records.Find(name.c_str(), src_only));
	}

	SourceRecords(pkgSourceList* list) : records(*list){};
};
