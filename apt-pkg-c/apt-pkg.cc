#include <cstddef>
#include <iostream>
#include <iterator>
#include <sstream>
#include <cstdint>

#include <assert.h>

#include <apt-pkg/configuration.h>
#include <apt-pkg/depcache.h>
#include <apt-pkg/sourcelist.h>
#include <apt-pkg/cachefile.h>
#include <apt-pkg/indexfile.h>
#include <apt-pkg/pkgcache.h>
#include <apt-pkg/pkgrecords.h>
#include <apt-pkg/version.h>
#include <apt-pkg/algorithms.h>

#include <apt-pkg/init.h>
#include <apt-pkg/pkgsystem.h>
#include <apt-pkg/policy.h>
#include <string>
// For Development Typing
#include "cxx-typing.h"
// Headers for the cxx bridge
#include "rust/cxx.h"
#include "rust-apt/src/raw.rs"
#include "apt-pkg.h"

struct PCache {
	// Owned by us.
	pkgCacheFile *cache_file;

	// Borrowed from cache_file.
	pkgCache *cache;

	// Borrowed from cache_file.
	pkgDepCache *depcache;

	// Owned by us.
	pkgRecords *records;
};

struct PPkgIterator {
	// Owned by us.
	pkgCache::PkgIterator iterator;

	// Borrow of "static" PCache.
//	PCache *pcache;
};

struct PVerIterator {
	// Owned by us.
	pkgCache::VerIterator iterator;

	// Borrowed from PCache.
	pkgCache::PkgIterator *pkg;

	// Borrow of "static" PCache.
//	PCache *cache;
};

// struct PDepIterator {
//	// Owned by us.
//	pkgCache::DepIterator iterator;

//	// Borrowed from PCache.
//	pkgCache::VerIterator *ver;

//	// Borrow of "static" PCache.
//	PCache *cache;
// };

struct PVerFileIterator {
	// Owned by us.
	pkgCache::VerFileIterator iterator;

	// Borrow of "static" PCache.
//	PCache *cache;
};

struct PPkgFileIterator {
	// Owned by us.
	pkgCache::PkgFileIterator iterator;
};

struct PVerFileParser {
	pkgRecords::Parser &parser;
};

// struct PDescFileIterator {

// };

// CXX Test Function
// int greet(rust::Str greetee) {
//   std::cout << "Hello, " << greetee << std::endl;
//   return get_num();
// }

const char *to_c_string(std::string s) {
	char *cstr = new char[s.length()+1];
	std::strcpy(cstr, s.c_str());
	return cstr;
}

void init_config_system() {
	pkgInitConfig(*_config);
	pkgInitSystem(*_config, _system);
}

// pkgCacheFile *get_cache_file() {
// 	return new pkgCacheFile();
// }

// pkgCache *get_cache(pkgCacheFile *cache_file) {
// 	return cache_file->GetPkgCache();
// }

// pkgRecords *get_records(pkgCache *cache) {
// 	return new pkgRecords(*cache);
// }

// pkgDepCache *get_depcache(pkgCacheFile *cache_file) {
// 	return cache_file->GetDepCache();
// }

void depcache_init(PCache *pcache) {
	pcache->depcache->Init(0);
	pkgApplyStatus(*pcache->depcache);
}

PCache *pkg_cache_create() {
	pkgCacheFile *cache_file = new pkgCacheFile();
	pkgCache *cache = cache_file->GetPkgCache();
	pkgRecords *records = new pkgRecords(*cache);
	pkgDepCache *depcache = cache_file->GetDepCache();

	PCache *ret = new PCache();
	cache_file->BuildSourceList();
	//std::unique_ptr<pkgSourceList> SrcList


	ret->cache_file = cache_file;
	ret->cache = cache;
	ret->records = records;
	ret->depcache = depcache;
// Initializing the depcache slows us down.
// Might not want to unless we actually need it.
//	depcache_init(ret);
	return ret;
}

bool pkg_is_upgradable(PCache *cache, PPkgIterator *wrapper) {
	pkgCache::PkgIterator &pkg = wrapper->iterator;
	if (pkg.CurrentVer() == 0) { return false; }
	return (*cache->cache_file)[pkg].Upgradable();
}

// pkgCache and pkgDepCache are cleaned up with cache_file.
void pkg_cache_release(PCache *cache) {
	delete cache->records;
	delete cache->cache_file;
	delete cache;
}

int32_t pkg_cache_compare_versions(PCache *cache, const char *left, const char *right) {
	// an int is returned here; presumably it will always be -1, 0 or 1.
	return cache->cache->VS->DoCmpVersion(left, left+strlen(left), right, right+strlen(right));
}

PPkgIterator *pkg_begin(PCache *pcache) {
	PPkgIterator *wrapper = new PPkgIterator();
	wrapper->iterator = pcache->cache->PkgBegin();
//	wrapper->cache = cache;
	return wrapper;
	}

PPkgIterator *pkg_cache_find_name(PCache *pcache, const char *name) {
	PPkgIterator *wrapper = new PPkgIterator();
	wrapper->iterator = pcache->cache->FindPkg(name);
	return wrapper;
}

PPkgIterator*pkg_cache_find_name_arch(PCache *pcache, const char *name, const char *arch) {
	PPkgIterator *wrapper = new PPkgIterator();
	wrapper->iterator = pcache->cache->FindPkg(name, arch);
	return wrapper;
}

PPkgIterator *pkg_clone(PPkgIterator *iterator) {
	PPkgIterator *wrapper = new PPkgIterator();
	wrapper->iterator = iterator->iterator;
	return wrapper;
}

void pkg_release(PPkgIterator *wrapper) {
	delete wrapper;
}

void pkg_next(PPkgIterator *wrapper) {
	++wrapper->iterator;
}

void ver_next(PVerIterator *wrapper) {
	++wrapper->iterator;
}

bool pkg_end(PPkgIterator *wrapper) {
	return wrapper->iterator.end();
}

bool ver_end(PVerIterator *wrapper) {
	return wrapper->iterator.end();
}

bool pkg_has_versions(PPkgIterator *wrapper) {
	return wrapper->iterator.VersionList().end() == false;
}

bool pkg_has_provides(PPkgIterator *wrapper) {
	return wrapper->iterator.ProvidesList().end() == false;
}

const char *pkg_name(PPkgIterator *wrapper) {
	return wrapper->iterator.Name();
}

rust::string get_fullname(PPkgIterator *wrapper, bool pretty) {
	return wrapper->iterator.FullName(pretty);
}

const char *pkg_arch(PPkgIterator *wrapper) {
	return wrapper->iterator.Arch();
}

PVerIterator *pkg_current_version(PPkgIterator *wrapper) {
	PVerIterator *new_wrapper = new PVerIterator();
	new_wrapper->iterator = wrapper->iterator.CurrentVer();
	new_wrapper->pkg = &wrapper->iterator;
//	new_wrapper->cache = wrapper->pcache;
	return new_wrapper;
}

PVerIterator *pkg_candidate_version(PCache *cache, PPkgIterator *wrapper) {
	PVerIterator *new_wrapper = new PVerIterator();
	new_wrapper->iterator = cache->cache_file->GetPolicy()->GetCandidateVer(wrapper->iterator);
	new_wrapper->pkg = &wrapper->iterator;
//	new_wrapper->cache = wrapper->pcache;
	return new_wrapper;
}

PVerIterator *pkg_version_list(PPkgIterator *wrapper) {
	PVerIterator *new_wrapper = new PVerIterator();
	new_wrapper->iterator = wrapper->iterator.VersionList();
	new_wrapper->pkg = &wrapper->iterator;
//	new_wrapper->cache = wrapper->pcache;
	return new_wrapper;
}

void ver_release(PVerIterator *wrapper) {
	delete wrapper;
}

const char *ver_str(PVerIterator *wrapper) {
	return wrapper->iterator.VerStr();
}

const char *ver_section(PVerIterator *wrapper) {
   return wrapper->iterator.Section();
}

const char *ver_priority_str(PVerIterator *wrapper) {
	return wrapper->iterator.PriorityType();
}

const char *ver_source_package(PVerIterator *wrapper) {
	return wrapper->iterator.SourcePkgName();
}

const char *ver_source_version(PVerIterator *wrapper) {
	return wrapper->iterator.SourceVerStr();
}

// #define VALIDATE_ITERATOR(I) {
// 	if ((I).Cache() != &depcache->GetCache()) return(false);
// 	return(true); }

template<typename Iterator>
static bool _validate(Iterator iter, pkgDepCache *depcache) {
	if (iter.Cache() != &depcache->GetCache())
	{return false;} else {return true;}
}

bool validate(PVerIterator *wrapper, PCache *pcache) {
	// if (wrapper->iterator.Cache() != &pcache->depcache->GetCache())
	// {return false;} else {return true;}
	return _validate(wrapper->iterator, pcache->depcache);
}

// bool validate(PVerIterator *wrapper, PCache *pcache) {
// 	if (wrapper->iterator.Cache() != &pcache->depcache->GetCache())
// 	{return false;} else {return true;}
// }

int32_t ver_priority(PCache *pcache, PVerIterator *wrapper) {
	// The priority is a "short", which is roughly a (signed) int16_t;
	// going bigger just in case
	pkgCache::VerIterator &ver = wrapper->iterator;
	return pcache->cache_file->GetPolicy()->GetPriority(ver);
}

const char *ver_arch(PVerIterator *wrapper) {
	return wrapper->iterator.Arch();
}

// PDepIterator *ver_iter_dep_iter(PVerIterator *wrapper) {
// 	PDepIterator *new_wrapper = new PDepIterator();
// 	new_wrapper->iterator = wrapper->iterator.DependsList();
// //	new_wrapper->cache = wrapper->cache;
// 	return new_wrapper;
// }

// void dep_iter_release(PDepIterator *wrapper) {
// 	delete wrapper;
// }

// void dep_iter_next(PDepIterator *wrapper) {
// 	++wrapper->iterator;
// }

// bool dep_iter_end(PDepIterator *wrapper) {
// 	return wrapper->iterator.end();
// }

// PPkgIterator *dep_iter_target_pkg(PDepIterator *wrapper) {
// 	PPkgIterator *new_wrapper = new PPkgIterator();
// 	new_wrapper->iterator = wrapper->iterator.TargetPkg();
// //	new_wrapper->cache = wrapper->cache;
// 	return new_wrapper;
// }

// const char *dep_iter_target_ver(PDepIterator *wrapper) {
// 	return wrapper->iterator.TargetVer();
// }

// const char *dep_iter_comp_type(PDepIterator *wrapper) {
// 	return wrapper->iterator.CompType();
// }

// const char *dep_iter_dep_type(PDepIterator *wrapper) {
// 	return wrapper->iterator.DepType();
// }

PVerFileIterator *ver_file(PVerIterator *wrapper) {
	PVerFileIterator *new_wrapper = new PVerFileIterator();
	new_wrapper->iterator = wrapper->iterator.FileList();
	return new_wrapper;
}

void ver_file_release(PVerFileIterator *wrapper) {
	delete wrapper;
}

void ver_file_next(PVerFileIterator *wrapper) {
	++wrapper->iterator;
}

bool ver_file_end(PVerFileIterator *wrapper) {
	return wrapper->iterator.end();
}

pkgRecords::Parser *ver_file_lookup(PCache *pcache, PVerFileIterator *wrapper) {
	return &pcache->records->Lookup(wrapper->iterator);
}

PPkgFileIterator *ver_pkg_file(PVerFileIterator *wrapper) {
	PPkgFileIterator *new_wrapper = new PPkgFileIterator();
	new_wrapper->iterator = wrapper->iterator.File();
	return new_wrapper;
}

pkgIndexFile *get_index_file(PCache *pcache, PPkgFileIterator *pkg_file) {
	//pkgCache *cache = pkg_file->iterator.Cache();
	pkgSourceList *SrcList = pcache->cache_file->GetSourceList();
	pkgIndexFile *Index;
	SrcList->FindIndex(pkg_file->iterator, Index);
	return Index;
}

const char *ver_uri(PCache *pcache, pkgRecords::Parser *parser, PPkgFileIterator *file) {
	pkgCache::PkgFileIterator pkg_file = file->iterator;
	pkgSourceList *SrcList = pcache->cache_file->GetSourceList();
	pkgIndexFile *Index;

	// Make sure we get the /dpkg/status file. Although I'd like to find a way not to include this one.
	if (SrcList->FindIndex(pkg_file, Index) == false) { _system->FindIndex(pkg_file, Index);}

	//std::cout << " " << Index->ArchiveURI(parser->FileName()) << "\n";
	return to_c_string(Index->ArchiveURI(parser->FileName()));
}
// Look at ver_uri for answers.
// Maybe *get_hash(idk, *for_real) -> hash {
	// std::cout << "SHA256 = ";
	// auto hashes = parser->Hashes();
	// auto hash = hashes.find("sha256");
	// if (hash == NULL) {std::cout << "NULL";} else {std::cout << parser->Hashes().find("sha256")->HashValue();}
// }


// const char *ver_file_parser_short_desc(PVerFileParser *parser) {
// 	std::string desc = parser->parser->ShortDesc();
// 	return to_c_string(desc);
// }

// const char *ver_file_parser_long_desc(PVerFileParser *parser) {
// 	std::string desc = parser->parser->LongDesc();
// 	return to_c_string(desc);
// }



// PCache.cache_file, Pcache.records(cachefile), ppkgIterator.iterator
//const char *GetLongDescription(pkgCacheFile &CacheFile, pkgRecords &records, pkgCache::PkgIterator P) {
rust::string long_desc(PCache *cache, PPkgIterator *wrapper) {
	pkgCache::PkgIterator P;
	//pkgCacheFile CacheFile;
	//pkgRecords & records;
	P = wrapper->iterator;
	pkgCacheFile * CacheFile = cache->cache_file;
	pkgRecords * records = cache->records;

	pkgPolicy *policy = CacheFile->GetPolicy();

	pkgCache::VerIterator ver;
	if (P->CurrentVer != 0)
		ver = P.CurrentVer();
	else
		ver = policy->GetCandidateVer(P);

	std::string const EmptyDescription = "(none)";
	if(ver.end() == true)
		return EmptyDescription;

	pkgCache::DescIterator const Desc = ver.TranslatedDescription();
	if (Desc.end() == false)
	{
		pkgRecords::Parser &parser = records->Lookup(Desc.FileList());

		std::string const longdesc = parser.LongDesc();
		if (longdesc.empty() == false)
		return SubstVar(longdesc, "\n ", "\n  ");
	}
	return EmptyDescription;
}

// const char *ver_file_parser_maintainer(PVerFileParser *parser) {
// 	std::string maint = parser->parser->Maintainer();
// 	return to_c_string(maint);
// }

// const char *ver_file_parser_homepage(PVerFileParser *parser) {
// 	std::string hp = parser->parser->Homepage();
// 	return to_c_string(hp);
// }

void pkg_file_iter_release(PPkgFileIterator *wrapper) {
	delete wrapper;
}

void pkg_file_iter_next(PPkgFileIterator *wrapper) {
	++wrapper->iterator;
}

bool pkg_file_iter_end(PPkgFileIterator *wrapper) {
	return wrapper->iterator.end();
}

const char *pkg_file_iter_file_name(PPkgFileIterator *wrapper) {
	return wrapper->iterator.FileName();
}

const char *pkg_file_iter_archive(PPkgFileIterator *wrapper) {
	return wrapper->iterator.Archive();
}

const char *pkg_file_iter_version(PPkgFileIterator *wrapper) {
	return wrapper->iterator.Version();
}

const char *pkg_file_iter_origin(PPkgFileIterator *wrapper) {
	return wrapper->iterator.Origin();
}

const char *pkg_file_iter_codename(PPkgFileIterator *wrapper) {
	return wrapper->iterator.Codename();
}

const char *pkg_file_iter_label(PPkgFileIterator *wrapper) {
	return wrapper->iterator.Label();
}

const char *pkg_file_iter_site(PPkgFileIterator *wrapper) {
	return wrapper->iterator.Site();
}

const char *pkg_file_iter_component(PPkgFileIterator *wrapper) {
	return wrapper->iterator.Component();
}

const char *pkg_file_iter_architecture(PPkgFileIterator *wrapper) {
	return wrapper->iterator.Architecture();
}

const char *pkg_file_iter_index_type(PPkgFileIterator *wrapper) {
	return wrapper->iterator.IndexType();
}
