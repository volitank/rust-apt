#pragma once
#include <apt-pkg/cachefile.h>
#include <apt-pkg/policy.h>
#include <memory>
#include "util.h"

#include "types.h"

struct VerIterator;
struct PkgIterator;

struct DepIterator : public pkgCache::DepIterator {
	void raw_next() { (*this)++; }

	UniquePtr<DepIterator> unique() const { return std::make_unique<DepIterator>(*this); }

	u8 dep_type() const { return (*this)->Type; }
	str comp_type() const { return handle_str(this->CompType()); }
	str target_ver() const { return handle_str(this->TargetVer()); }

	inline bool or_dep() const {
		return ((*this)->CompareOp & pkgCache::Dep::Or) == pkgCache::Dep::Or;
	}

	UniquePtr<PkgIterator> parent_pkg() const;
	UniquePtr<PkgIterator> target_pkg() const;
	UniquePtr<VerIterator> parent_ver() const;
	UniquePtr<std::vector<VerIterator>> all_targets() const;

	DepIterator(const pkgCache::DepIterator& base) : pkgCache::DepIterator(base){};
};

struct PrvIterator : public pkgCache::PrvIterator {
	void raw_next() { (*this)++; }

	str name() const { return this->Name(); }
	str version_str() const { return handle_str(this->ProvideVersion()); }

	UniquePtr<PkgIterator> target_pkg() const;
	UniquePtr<VerIterator> target_ver() const;

	UniquePtr<PrvIterator> unique() const { return std::make_unique<PrvIterator>(*this); }

	PrvIterator(const pkgCache::PrvIterator& base) : pkgCache::PrvIterator(base){};
};

struct PkgFileIterator : public pkgCache::PkgFileIterator {
	void raw_next() { (*this)++; }

	str filename() const { return handle_str(this->FileName()); }
	str archive() const { return handle_str(this->Archive()); }
	str origin() const { return handle_str(this->Origin()); }
	str codename() const { return handle_str(this->Codename()); }
	str label() const { return handle_str(this->Label()); }
	str site() const { return handle_str(this->Site()); }
	str component() const { return handle_str(this->Component()); }
	str arch() const { return handle_str(this->Architecture()); }
	str index_type() const { return handle_str(this->IndexType()); }

	bool is_downloadable() const { return !this->Flagged(pkgCache::Flag::NotSource); }

	UniquePtr<PkgFileIterator> unique() const { return std::make_unique<PkgFileIterator>(*this); }

	PkgFileIterator(const pkgCache::PkgFileIterator& base) : pkgCache::PkgFileIterator(base){};
};

struct VerFileIterator : public pkgCache::VerFileIterator {
	void raw_next() { (*this)++; }

	UniquePtr<VerFileIterator> unique() const { return std::make_unique<VerFileIterator>(*this); }

	UniquePtr<PkgFileIterator> package_file() const {
		return std::make_unique<PkgFileIterator>(this->File());
	};

	VerFileIterator(const pkgCache::VerFileIterator& base) : pkgCache::VerFileIterator(base){};
};

struct DescIterator : public pkgCache::DescIterator {
	void raw_next() { (*this)++; }

	UniquePtr<DescIterator> unique() const { return std::make_unique<DescIterator>(*this); }

	DescIterator(const pkgCache::DescIterator& base) : pkgCache::DescIterator(base){};
};

struct VerIterator : public pkgCache::VerIterator {
	void raw_next() { (*this)++; }

	str version() const { return this->VerStr(); }
	str arch() const { return this->Arch(); }
	str section() const { return handle_str(this->Section()); }
	str priority_str() const { return handle_str(this->PriorityType()); }
	str source_name() const { return this->SourcePkgName(); }
	str source_version() const { return this->SourceVerStr(); }
	u64 size() const { return (*this)->Size; }
	u64 installed_size() const { return (*this)->InstalledSize; }
	// TODO: Move this into rust?
	bool is_installed() const { return this->ParentPkg().CurrentVer() == *this; }

	UniquePtr<PkgIterator> parent_pkg() const;

	// This is for backend records lookups.
	UniquePtr<DescIterator> translated_desc() const {
		return std::make_unique<DescIterator>(this->TranslatedDescription());
	}

	// This is for backend records lookups.
	// You go through here to get the package files.
	UniquePtr<VerFileIterator> version_files() const {
		return std::make_unique<VerFileIterator>(this->FileList());
	}

	UniquePtr<DepIterator> depends() const {
		return std::make_unique<DepIterator>(this->DependsList());
	}

	UniquePtr<PrvIterator> provides() const {
		return std::make_unique<PrvIterator>(this->ProvidesList());
	}

	UniquePtr<VerIterator> unique() const { return std::make_unique<VerIterator>(*this); }

	VerIterator(const pkgCache::VerIterator& base) : pkgCache::VerIterator(base){};
};

struct PkgIterator : public pkgCache::PkgIterator {
	void raw_next() { (*this)++; }

	str name() const { return this->Name(); }
	str arch() const { return this->Arch(); }
	String fullname(bool Pretty) const { return this->FullName(Pretty); }
	u8 current_state() const { return (*this)->CurrentState; }
	u8 inst_state() const { return (*this)->InstState; }
	u8 selected_state() const { return (*this)->SelectedState; }

	/// True if the package is essential.
	bool is_essential() const { return ((*this)->Flags & pkgCache::Flag::Essential) != 0; }

	UniquePtr<VerIterator> current_version() const {
		return std::make_unique<VerIterator>(this->CurrentVer());
	}

	UniquePtr<VerIterator> versions() const {
		return std::make_unique<VerIterator>(this->VersionList());
	}

	UniquePtr<PrvIterator> provides() const {
		return std::make_unique<PrvIterator>(this->ProvidesList());
	}

	UniquePtr<DepIterator> rdepends() const {
		return std::make_unique<DepIterator>(this->RevDependsList());
	}

	UniquePtr<PkgIterator> unique() const { return std::make_unique<PkgIterator>(*this); }

	PkgIterator(const pkgCache::PkgIterator& base) : pkgCache::PkgIterator(base){};
};

inline UniquePtr<PkgIterator> PrvIterator::target_pkg() const {
	return std::make_unique<PkgIterator>(this->OwnerPkg());
}

inline UniquePtr<VerIterator> PrvIterator::target_ver() const {
	return std::make_unique<VerIterator>(this->OwnerVer());
}

inline UniquePtr<PkgIterator> DepIterator::parent_pkg() const {
	return std::make_unique<PkgIterator>(this->ParentPkg());
}

inline UniquePtr<VerIterator> DepIterator::parent_ver() const {
	return std::make_unique<VerIterator>(this->ParentVer());
}

inline UniquePtr<PkgIterator> DepIterator::target_pkg() const {
	return std::make_unique<PkgIterator>(this->TargetPkg());
}

inline UniquePtr<std::vector<VerIterator>> DepIterator::all_targets() const {
	// pkgPrioSortList for sorting by priority?
	//
	// The version list returned is not a VerIterator.
	// They are the lowest level Version structs. We need to iter these
	// Convert them into our VerIterator, and then we can handle that in rust.
	UniquePtr<pkgCache::Version*[]> VList(this->AllTargets());
	std::vector<VerIterator> list;

	for (pkgCache::Version** I = VList.get(); *I != 0; ++I) {
		list.push_back(VerIterator(pkgCache::VerIterator(*this->Cache(), *I)));
	}

	return std::make_unique<std::vector<VerIterator>>(list);
}

inline UniquePtr<PkgIterator> VerIterator::parent_pkg() const {
	return std::make_unique<PkgIterator>(this->ParentPkg());
}
