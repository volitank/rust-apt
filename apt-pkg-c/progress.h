#pragma once
#include "rust/cxx.h"
#include <apt-pkg/acquire-item.h>

struct Worker;

class DynUpdateProgress {
	public:
	DynUpdateProgress(DynUpdateProgress&&) noexcept;
	~DynUpdateProgress() noexcept;
	using IsRelocatable = std::true_type;

	int pulse_interval() const noexcept;
	void hit(u_int32_t id, std::string description) const noexcept;
	void fetch(u_int32_t id, std::string description, u_int64_t file_size) const noexcept;
	void fail(u_int32_t id, std::string description, u_int32_t status, std::string error_text) const noexcept;
	void pulse(rust::vec<Worker> workers,
	double percent,
	u_int64_t total_bytes,
	u_int64_t current_bytes,
	u_int64_t current_cps) const noexcept;
	void done() const noexcept;
	void start() const noexcept;
	void stop(u_int64_t fetched_bytes,
	u_int64_t elapsed_time,
	u_int64_t current_cps,
	bool pending_errors) const noexcept;
};

class AcqTextStatus : public pkgAcquireStatus {
	unsigned long ID;
	/// Callback to the rust struct
	DynUpdateProgress& callback;

	void clearLastLine();
	void AssignItemID(pkgAcquire::ItemDesc& Itm);

	public:
	virtual bool ReleaseInfoChanges(metaIndex const* const LastRelease,
	metaIndex const* const CurrentRelease,
	std::vector<ReleaseInfoChange>&& Changes);
	virtual bool MediaChange(std::string Media, std::string Drive);
	virtual void IMSHit(pkgAcquire::ItemDesc& Itm);
	virtual void Fetch(pkgAcquire::ItemDesc& Itm);
	virtual void Done(pkgAcquire::ItemDesc& Itm);
	virtual void Fail(pkgAcquire::ItemDesc& Itm);
	virtual void Start();
	virtual void Stop();

	bool Pulse(pkgAcquire* Owner);

	AcqTextStatus(DynUpdateProgress& callback);
};
