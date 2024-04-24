#pragma once
#include <apt-pkg/cachefile.h>
#include <apt-pkg/policy.h>
#include <memory>
#include "rust/cxx.h"
#include "util.h"

struct VerIterator;
struct PkgIterator;

class DepIterator : public pkgCache::DepIterator {
   public:
	void raw_next() { (*this)++; }

	std::unique_ptr<DepIterator> unique() const { return std::make_unique<DepIterator>(*this); }

	uint8_t u8_dep_type() const { return (*this)->Type; }
	rust::str comp_type() const { return handle_str(this->CompType()); }
	rust::str target_ver() const { return handle_str(this->TargetVer()); }

	inline bool compare_op() const {
		return ((*this)->CompareOp & pkgCache::Dep::Or) == pkgCache::Dep::Or;
	}

	std::unique_ptr<PkgIterator> parent_pkg() const;
	std::unique_ptr<PkgIterator> target_pkg() const;
	std::unique_ptr<VerIterator> parent_ver() const;
	std::unique_ptr<std::vector<VerIterator>> all_targets() const;

	DepIterator(const pkgCache::DepIterator& base) : pkgCache::DepIterator(base){};
};

class PrvIterator : public pkgCache::PrvIterator {
   public:
	void raw_next() { (*this)++; }

	rust::str name() const { return this->Name(); }
	rust::str version_str() const { return handle_str(this->ProvideVersion()); }

	std::unique_ptr<PkgIterator> target_pkg() const;
	std::unique_ptr<VerIterator> target_ver() const;

	std::unique_ptr<PrvIterator> unique() const { return std::make_unique<PrvIterator>(*this); }

	PrvIterator(const pkgCache::PrvIterator& base) : pkgCache::PrvIterator(base){};
};

class PkgFileIterator : public pkgCache::PkgFileIterator {
   public:
	void raw_next() { (*this)++; }

	rust::str filename() const { return handle_str(this->FileName()); }
	rust::str archive() const { return handle_str(this->Archive()); }
	rust::str origin() const { return handle_str(this->Origin()); }
	rust::str codename() const { return handle_str(this->Codename()); }
	rust::str label() const { return handle_str(this->Label()); }
	rust::str site() const { return handle_str(this->Site()); }
	rust::str component() const { return handle_str(this->Component()); }
	rust::str arch() const { return handle_str(this->Architecture()); }
	rust::str index_type() const { return handle_str(this->IndexType()); }

	std::unique_ptr<PkgFileIterator> unique() const {
		return std::make_unique<PkgFileIterator>(*this);
	}

	PkgFileIterator(const pkgCache::PkgFileIterator& base) : pkgCache::PkgFileIterator(base){};
};

class VerFileIterator : public pkgCache::VerFileIterator {
   public:
	void raw_next() { (*this)++; }

	std::unique_ptr<VerFileIterator> unique() const {
		return std::make_unique<VerFileIterator>(*this);
	}

	std::unique_ptr<PkgFileIterator> pkg_file() const {
		return std::make_unique<PkgFileIterator>(this->File());
	};

	VerFileIterator(const pkgCache::VerFileIterator& base) : pkgCache::VerFileIterator(base){};
};

class DescFileIterator : public pkgCache::DescFileIterator {
   public:
	void raw_next() { (*this)++; }

	std::unique_ptr<DescFileIterator> unique() const {
		return std::make_unique<DescFileIterator>(*this);
	}

	std::unique_ptr<PkgFileIterator> pkg_file() const {
		return std::make_unique<PkgFileIterator>(this->File());
	};

	DescFileIterator(const pkgCache::DescFileIterator& base) : pkgCache::DescFileIterator(base){};
};

class VerIterator : public pkgCache::VerIterator {
   public:
	void raw_next() { (*this)++; }

	rust::str version() const { return this->VerStr(); }
	rust::str arch() const { return this->Arch(); }
	rust::str section() const { return handle_str(this->Section()); }
	rust::str priority_str() const { return handle_str(this->PriorityType()); }
	rust::str source_name() const { return this->SourcePkgName(); }
	rust::str source_version() const { return this->SourceVerStr(); }
	uint32_t id() const { return (*this)->ID; }
	uint64_t size() const { return (*this)->Size; }
	uint64_t installed_size() const { return (*this)->InstalledSize; }
	// TODO: Move this into rust?
	bool is_installed() const { return this->ParentPkg().CurrentVer() == *this; }

	std::unique_ptr<PkgIterator> parent_pkg() const;

	// This is for backend records lookups. You can also get package files from here.
	std::unique_ptr<DescFileIterator> u_description_file() const {
		auto desc_file = this->TranslatedDescription();
		// Must check if DescFileIterator is null first.
		// See https://gitlab.com/volian/rust-apt/-/issues/28
		if (desc_file.end()) { throw std::runtime_error("DescFile doesn't exist"); }

		return std::make_unique<DescFileIterator>(desc_file.FileList());
	}

	// You go through here to get the package files.
	std::unique_ptr<VerFileIterator> u_version_file() const {
		return std::make_unique<VerFileIterator>(this->FileList());
	}

	std::unique_ptr<DepIterator> u_depends() const {
		return std::make_unique<DepIterator>(this->DependsList());
	}

	std::unique_ptr<PrvIterator> u_provides() const {
		return std::make_unique<PrvIterator>(this->ProvidesList());
	}

	std::unique_ptr<VerIterator> unique() const { return std::make_unique<VerIterator>(*this); }

	VerIterator(const pkgCache::VerIterator& base) : pkgCache::VerIterator(base){};
};

class PkgIterator : public pkgCache::PkgIterator {
   public:
	void raw_next() { (*this)++; }

	rust::str name() const { return this->Name(); }
	rust::str arch() const { return this->Arch(); }
	rust::string fullname(bool Pretty) const { return this->FullName(Pretty); }
	u_int32_t id() const { return (*this)->ID; }
	u_int8_t current_state() const { return (*this)->CurrentState; }
	u_int8_t inst_state() const { return (*this)->InstState; }
	u_int8_t selected_state() const { return (*this)->SelectedState; }

	/// True if the package is essential.
	bool is_essential() const { return ((*this)->Flags & pkgCache::Flag::Essential) != 0; }

	std::unique_ptr<VerIterator> u_current_version() const {
		return std::make_unique<VerIterator>(this->CurrentVer());
	}

	std::unique_ptr<VerIterator> u_version_list() const {
		return std::make_unique<VerIterator>(this->VersionList());
	}

	std::unique_ptr<PrvIterator> u_provides() const {
		return std::make_unique<PrvIterator>(this->ProvidesList());
	}

	std::unique_ptr<DepIterator> u_rev_depends() const {
		return std::make_unique<DepIterator>(this->RevDependsList());
	}

	std::unique_ptr<PkgIterator> unique() const { return std::make_unique<PkgIterator>(*this); }

	PkgIterator(const pkgCache::PkgIterator& base) : pkgCache::PkgIterator(base){};
};

inline std::unique_ptr<PkgIterator> PrvIterator::target_pkg() const {
	return std::make_unique<PkgIterator>(this->OwnerPkg());
}

inline std::unique_ptr<VerIterator> PrvIterator::target_ver() const {
	return std::make_unique<VerIterator>(this->OwnerVer());
}

inline std::unique_ptr<PkgIterator> DepIterator::parent_pkg() const {
	return std::make_unique<PkgIterator>(this->ParentPkg());
}

inline std::unique_ptr<VerIterator> DepIterator::parent_ver() const {
	return std::make_unique<VerIterator>(this->ParentVer());
}

inline std::unique_ptr<PkgIterator> DepIterator::target_pkg() const {
	return std::make_unique<PkgIterator>(this->TargetPkg());
}

inline std::unique_ptr<std::vector<VerIterator>> DepIterator::all_targets() const {
	// pkgPrioSortList for sorting by priority?
	//
	// The version list returned is not a VerIterator.
	// They are the lowest level Version structs. We need to iter these
	// Convert them into our VerIterator, and then we can handle that in rust.
	std::unique_ptr<pkgCache::Version*[]> VList(this->AllTargets());
	std::vector<VerIterator> list;

	for (pkgCache::Version** I = VList.get(); *I != 0; ++I) {
		list.push_back(VerIterator(pkgCache::VerIterator(*this->Cache(), *I)));
	}

	return std::make_unique<std::vector<VerIterator>>(list);
}

inline std::unique_ptr<PkgIterator> VerIterator::parent_pkg() const {
	return std::make_unique<PkgIterator>(this->ParentPkg());
}
