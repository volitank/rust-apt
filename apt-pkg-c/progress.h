#pragma once
#include <apt-pkg/install-progress.h>
#include <apt-pkg/progress.h>
#include "rust/cxx.h"

#include "rust-apt/src/progress.rs"

struct OpProgressWrapper : public OpProgress {
	/// Callback to the rust struct
	OperationProgress& callback;

	void Update() { callback.update(Op, Percent); };
	void Done() { callback.done(); };

	OpProgressWrapper(OperationProgress& callback) : callback(callback){};
};

struct PackageManagerWrapper : public APT::Progress::PackageManagerFancy {
	/// Callback to the rust struct
	InstallProgress& callback;

	bool StatusChanged(
		std::string pkgname,
		unsigned int steps_done,
		unsigned int total_steps,
		std::string action
	) {
		callback.status_changed(pkgname, steps_done, total_steps, action);
		return true;
	};

	void Error(
		std::string pkgname,
		unsigned int steps_done,
		unsigned int total_steps,
		std::string error
	) {
		callback.error(pkgname, steps_done, total_steps, error);
	};

	PackageManagerWrapper(InstallProgress& callback) : callback(callback){};
};
