#pragma once
#include <apt-pkg/acquire-item.h>
#include <apt-pkg/install-progress.h>
#include <apt-pkg/progress.h>
#include "rust/cxx.h"

struct Worker;

/// Classes for pkgAcquireStatus usage.
class DynAcquireProgress {
   public:
	DynAcquireProgress(DynAcquireProgress&&) noexcept;
	~DynAcquireProgress() noexcept;
	using IsRelocatable = std::true_type;

	int pulse_interval() const noexcept;
	void hit(u_int32_t id, std::string description) const noexcept;
	void fetch(u_int32_t id, std::string description, u_int64_t file_size) const noexcept;
	void fail(u_int32_t id, std::string description, u_int32_t status, std::string error_text)
		const noexcept;
	void pulse(
		rust::vec<Worker> workers,
		double percent,
		u_int64_t total_bytes,
		u_int64_t current_bytes,
		u_int64_t current_cps
	) const noexcept;
	void done() const noexcept;
	void start() const noexcept;
	void stop(
		u_int64_t fetched_bytes,
		u_int64_t elapsed_time,
		u_int64_t current_cps,
		bool pending_errors
	) const noexcept;
};

class AcqTextStatus : public pkgAcquireStatus {
	unsigned long ID;
	/// Callback to the rust struct
	DynAcquireProgress& callback;

	void clearLastLine();
	void AssignItemID(pkgAcquire::ItemDesc& Itm);

   public:
	virtual bool ReleaseInfoChanges(
		metaIndex const* const LastRelease,
		metaIndex const* const CurrentRelease,
		std::vector<ReleaseInfoChange>&& Changes
	);
	virtual bool MediaChange(std::string Media, std::string Drive);
	virtual void IMSHit(pkgAcquire::ItemDesc& Itm);
	virtual void Fetch(pkgAcquire::ItemDesc& Itm);
	virtual void Done(pkgAcquire::ItemDesc& Itm);
	virtual void Fail(pkgAcquire::ItemDesc& Itm);
	virtual void Start();
	virtual void Stop();

	bool Pulse(pkgAcquire* Owner);

	AcqTextStatus(DynAcquireProgress& callback);
};

/// Classes for OpProgress usage.
class DynOperationProgress {
   public:
	void op_update(std::string operation, float percent);
	void op_done();
};

class OpProgressWrapper : public OpProgress {
	/// Callback to the rust struct
	DynOperationProgress& callback;

   public:
	void Update();
	void Done();

	OpProgressWrapper(DynOperationProgress& callback);
};

/// Classes for InstallProgress usage.
class DynInstallProgress {
   public:
	/// This is supposed to return a bool, but I have zero clue when that'd be needed in practice.
	/// TODO: StatusChanged returns a bool sometimes in the C++ lib, though I'm not sure if it ever
	/// happens in practice.
	void inst_status_changed(
		std::string pkgname,
		u_int64_t steps_done,
		u_int64_t total_steps,
		std::string action
	);
	void inst_error(
		std::string pkgname,
		u_int64_t steps_done,
		u_int64_t total_steps,
		std::string error
	);
};

class PackageManagerWrapper : public APT::Progress::PackageManagerFancy {
	// class PackageManagerWrapper {
	/// Callback to the rust struct
	DynInstallProgress& callback;

   public:
	virtual bool StatusChanged(
		std::string pkgname,
		unsigned int steps_done,
		unsigned int total_steps,
		std::string action
	);
	virtual void Error(
		std::string pkgname,
		unsigned int steps_done,
		unsigned int total_steps,
		std::string error
	);

	PackageManagerWrapper(DynInstallProgress& callback);
};
