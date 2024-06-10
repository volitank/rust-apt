#pragma once
#include <apt-pkg/acquire-item.h>
#include <apt-pkg/acquire-worker.h>
#include <apt-pkg/acquire.h>
#include <iostream>
#include <memory>
#include "rust/cxx.h"

#include "rust-apt/src/progress.rs"

#include "types.h"

// ItemState Enum
using ItemState = pkgAcquire::Item::ItemState;

struct PkgAcquire {
	pkgAcquire* ptr;
	// If true, delete ptr during deconstruction
	bool del;

	UniquePtr<std::vector<ItemDesc>> uris() const;

	UniquePtr<std::vector<AcqWorker>> workers() const;

	PkgAcquire() : ptr(new pkgAcquire), del(true){};
	PkgAcquire(pkgAcquire* base) : ptr(base), del(false){};
	~PkgAcquire() {
		if (del) { delete ptr; }
	};
};

struct Item {
	pkgAcquire::Item* ptr;

	u32 id() const { return ptr->ID; }
	bool complete() const { return ptr->Complete; }
	u64 file_size() const { return ptr->FileSize; }
	ItemState status() const { return ptr->Status; }
	String uri() const { return ptr->DescURI(); }
	String dest_file() const { return ptr->DestFile; }
	String error_text() const { return ptr->ErrorText; }
	String active_subprocess() const { return ptr->ActiveSubprocess; }

	UniquePtr<PkgAcquire> owner() const { return std::make_unique<PkgAcquire>(ptr->GetOwner()); }

	Item(pkgAcquire::Item* base) : ptr(base){};
};

struct ItemDesc {
	pkgAcquire::ItemDesc* ptr;

	String uri() const { return ptr->URI; }
	String description() const { return ptr->Description; }
	String short_desc() const { return ptr->ShortDesc; }

	UniquePtr<Item> owner() const { return std::make_unique<Item>(ptr->Owner); }

	// Cast away the constness in this case. We aren't going to change it.
	ItemDesc(const pkgAcquire::ItemDesc* base) : ptr(const_cast<pkgAcquire::ItemDesc*>(base)){};
	ItemDesc(pkgAcquire::ItemDesc* base) : ptr(base){};
};

struct AcqWorker {
	pkgAcquire::Worker* ptr;
	pkgAcquire::ItemDesc* item_desc;

	String status() const { return ptr->Status; }
	u64 current_size() const { return ptr->CurrentItem->CurrentSize; }
	u64 total_size() const { return ptr->CurrentItem->TotalSize; }

	UniquePtr<ItemDesc> item() const {
		if (ptr->CurrentItem == 0) { throw std::runtime_error("Null Item!"); }
		return std::make_unique<ItemDesc>(item_desc);
	}

	AcqWorker(pkgAcquire::Worker* base) : ptr(base), item_desc(base->CurrentItem){};
};

struct AcqTextStatus : public pkgAcquireStatus {
	u32 ID;
	/// Callback to the rust struct
	AcquireProgress* callback;

	void AssignItemID(pkgAcquire::ItemDesc& Itm) {
		if (Itm.Owner->ID == 0) Itm.Owner->ID = ID++;
	};

	bool ReleaseInfoChanges(
		metaIndex const* const LastRelease,
		metaIndex const* const CurrentRelease,
		std::vector<ReleaseInfoChange>&& Changes
	) {
		(void)LastRelease;
		(void)CurrentRelease;
		(void)Changes;
		// if (Quiet >= 2 || isatty(STDOUT_FILENO) != 1 || isatty(STDIN_FILENO) != 1 ||
		// _config->FindB("APT::Get::Update::InteractiveReleaseInfoChanges", false) == false)
		// 	return pkgAcquireStatus::ReleaseInfoChanges(nullptr, nullptr, std::move(Changes));

		// _error->PushToStack();
		// auto const confirmed = pkgAcquireStatus::ReleaseInfoChanges(L, N,
		// std::move(Changes)); if (confirmed == true) { _error->MergeWithStack();
		// 	return true;
		// }
		// clearLastLine();
		// _error->DumpErrors(out, GlobalError::NOTICE, false);
		// _error->RevertToStack();

		// return YnPrompt(_("Do you want to accept these changes and continue updating from this
		// repository?"), false, false, out, out);

		// Not yet implemented. Remove return true when it is.
		return true;
	};
	bool MediaChange(std::string Media, std::string Drive) {
		(void)Drive;
		(void)Media;
		// If we do not output on a terminal and one of the options to avoid user
		// interaction is given, we assume that no user is present who could react
		// on your media change request
		// if (isatty(STDOUT_FILENO) != 1 && Quiet >= 2 &&
		// (_config->FindB("APT::Get::Assume-Yes", false) == true ||
		// _config->FindB("APT::Get::Force-Yes", false) == true ||
		// _config->FindB("APT::Get::Trivial-Only", false) == true))

		// 	return false;

		// clearLastLine();
		// ioprintf(out,
		// "Media change: please insert the disc labeled\n"
		// " '%s'\n"
		// "in the drive '%s' and press [Enter]\n",
		// Media.c_str(), Drive.c_str());

		// char C = 0;
		// bool bStatus = true;
		// while (C != '\n' && C != '\r') {
		// 	int len = read(STDIN_FILENO, &C, 1);
		// 	if (C == 'c' || len <= 0) {
		// 		bStatus = false;
		// 		break;
		// 	}
		// }

		// if (bStatus) Update = true;
		// return bStatus;

		// I'm not sure what to return here.
		// Will need to test media swaps at some point
		return false;
	};

	void IMSHit(pkgAcquire::ItemDesc& Itm) {
		Update = true;
		AssignItemID(Itm);
		callback->hit(ItemDesc(&Itm));
	};

	void Fetch(pkgAcquire::ItemDesc& Itm) {
		Update = true;
		if (Itm.Owner->Complete == true) return;
		AssignItemID(Itm);
		callback->fetch(ItemDesc(&Itm));
	};

	void Done(pkgAcquire::ItemDesc& Itm) {
		Update = true;
		AssignItemID(Itm);
		callback->done(ItemDesc(&Itm));
	};

	void Fail(pkgAcquire::ItemDesc& Itm) {
		Update = true;
		AssignItemID(Itm);
		callback->fail(ItemDesc(&Itm));
	};

	void Start() {
		pkgAcquireStatus::Start();
		callback->start();
		ID = 1;
	};

	void Stop() {
		pkgAcquireStatus::Stop();
		callback->stop();
	};

	bool Pulse(pkgAcquire* Owner) {
		Update = true;
		pkgAcquireStatus::Pulse(Owner);
		callback->pulse(Owner);
		return true;
	};

	void set_callback(AcquireProgress* callback) { this->callback = callback; };

	u64 current_cps() const { return this->CurrentCPS; }
	u64 elapsed_time() const { return this->ElapsedTime; }
	u64 fetched_bytes() const { return this->FetchedBytes; }
	u64 current_bytes() const { return this->CurrentBytes; }
	u64 total_bytes() const { return this->TotalBytes; }
	f64 percent() const { return this->Percent; }

	AcqTextStatus() : pkgAcquireStatus(), callback(0){};
};

inline UniquePtr<std::vector<ItemDesc>> PkgAcquire::uris() const {
	std::vector<ItemDesc> list;

	pkgAcquire::UriIterator I = ptr->UriBegin();
	for (; I != ptr->UriEnd(); ++I) {
		list.push_back(ItemDesc(I.operator->()));
	}
	return std::make_unique<std::vector<ItemDesc>>(list);
}

inline UniquePtr<std::vector<AcqWorker>> PkgAcquire::workers() const {
	std::vector<AcqWorker> list;

	for (pkgAcquire::Worker* I = ptr->WorkersBegin(); I != 0; I = ptr->WorkerStep(I)) {
		list.push_back(I);
	}
	return std::make_unique<std::vector<AcqWorker>>(list);
}

inline UniquePtr<AcqTextStatus> acquire_status() { return std::make_unique<AcqTextStatus>(); }
inline UniquePtr<PkgAcquire> create_acquire() { return std::make_unique<PkgAcquire>(); }
