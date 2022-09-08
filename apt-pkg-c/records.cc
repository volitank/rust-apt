#include "rust-apt/src/records.rs"
#include "rust-apt/src/cache.rs"

// Helper function
//
/// Check if a string exists and return a Result to rust
static rust::string check_string(std::string string) {
	if (string.empty()) {
		throw std::runtime_error("String is empty");
	}
	return string;
}

/// Create the Package Records.
Records records_create(const std::unique_ptr<PkgCacheFile>& cache) {
	return Records{
		std::make_unique<PkgRecords>(cache->GetPkgCache()),
	};
}


/// Moves the Records into the correct place.
void ver_file_lookup(Records& records, const VersionFile& ver_file) {
	auto Index = ver_file.ptr->Index();
	if (records.records->last == Index) {
		return;
	}

	records.records->last = Index;
	records.records->parser = &records.records->records.Lookup(*ver_file.ptr);
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
const VersionFile& ver_file) {
	pkgSourceList* SrcList = cache->GetSourceList();
	pkgIndexFile* Index;

	// Need to come up with a way to make this lazy.
	// Initialize and store when we need it so multiple looks ups aren't necessary.
	//
	// We may want to implement the SourcesList separate from the cache.
	//
	// We could also pull this out into a special binding, find a way to return
	// The IndexFile and put it in a OnceCell in rust to avoid getting it several times.
	//
	// As of right now this also exists in cache.cc in is_trusted for the PackageFile.
	//
	// This is solved in the PackageFile by wrapping it on the C++ side.

	if (!SrcList->FindIndex(ver_file.ptr->File(), Index)) {
		_system->FindIndex(ver_file.ptr->File(), Index);
	}
	return Index->ArchiveURI(records.records->parser->FileName());
}


/// Return the translated long description of a Package.
rust::string long_desc(const Records& records) {
	return check_string(records.records->parser->LongDesc());
}

/// Return the translated short description of a Package.
rust::string short_desc(const Records& records) {
	return check_string(records.records->parser->ShortDesc());
}

/// Return the Source package version string.
rust::string get_field(const Records& records, rust::string field) {
	return check_string(records.records->parser->RecordField(field.c_str()));
}

/// Find the hash of a Version. Returns "KeyError" (lul python) if there is no hash.
rust::string hash_find(const Records& records, rust::string hash_type) {
	auto hashes = records.records->parser->Hashes();
	auto hash = hashes.find(hash_type.c_str());
	if (hash == NULL) {
		throw std::runtime_error("KeyError");
	}
	return hash->HashValue();
}
