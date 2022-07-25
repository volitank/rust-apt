#include "rust-apt/src/records.rs"
#include "rust-apt/src/cache.rs"

/// Create the Package Records.
Records records_create(const std::unique_ptr<PkgCacheFile>& cache) {
	return Records{
		std::make_unique<PkgRecords>(cache->GetPkgCache()),
	};
}


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
		throw std::runtime_error("KeyError");
	}
	return hash->HashValue();
}
