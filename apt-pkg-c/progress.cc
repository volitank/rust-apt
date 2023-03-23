#include "rust-apt/src/raw/progress.rs"
#include "rust-apt/apt-pkg-c/defines.h"
#include "progress.h"
#include <apt-pkg/acquire-worker.h>
#include <apt-pkg/error.h>

/// AcqTextStatus modeled from in apt-private/acqprogress.cc
///
/// AcqTextStatus::AcqTextStatus - Constructor
AcqTextStatus::AcqTextStatus(DynAcquireProgress& callback)
: pkgAcquireStatus(), callback(callback) {}


/// Called when progress has started.
///
/// We do not print anything here to remain consistent with apt.
/// lastline length is set to 0 to ensure consistency when progress begins.
void AcqTextStatus::Start() {
	pkgAcquireStatus::Start();
	start(callback);
	ID = 1;
}


/// Internal function to assign the correct ID to an Item.
///
/// We can likely move this into the rust side with a refactor of the items.
/// Not sure it that should be done, we'll see in the future.
void AcqTextStatus::AssignItemID(pkgAcquire::ItemDesc& Itm) {
	if (Itm.Owner->ID == 0) Itm.Owner->ID = ID++;
}


/// Called when an item is confirmed to be up-to-date.
///
/// Prints out the short description and the expected size.
void AcqTextStatus::IMSHit(pkgAcquire::ItemDesc& Itm) {

	AssignItemID(Itm);

	hit(callback, Itm.Owner->ID, Itm.Description);
	Update = true;
}


/// Called when an Item has started to download
///
/// Prints out the short description and the expected size.
void AcqTextStatus::Fetch(pkgAcquire::ItemDesc& Itm) {
	Update = true;
	if (Itm.Owner->Complete == true) return;

	AssignItemID(Itm);
	fetch(callback, Itm.Owner->ID, Itm.Description, Itm.Owner->FileSize);
}


/// Called when an item is successfully and completely fetched.
///
/// We don't print anything here to remain consistent with apt.
void AcqTextStatus::Done(pkgAcquire::ItemDesc& Itm) {
	Update = true;
	AssignItemID(Itm);
	done(callback);
}


/// Called when an Item fails to download.
///
/// Print out the ErrorText for the Item.
void AcqTextStatus::Fail(pkgAcquire::ItemDesc& Itm) {
	AssignItemID(Itm);

	fail(callback, Itm.Owner->ID, Itm.Description, Itm.Owner->Status, Itm.Owner->ErrorText);
	Update = true;
}


/// Called when progress has finished.
///
/// prints out the bytes downloaded and the overall average line speed.
void AcqTextStatus::Stop() {
	pkgAcquireStatus::Stop();

	stop(callback, FetchedBytes, ElapsedTime, CurrentCPS, _error->PendingError());
}


/// Called periodically to provide the overall progress information
///
/// Draws the current progress.
/// Each line has an overall percent meter and a per active item status
/// meter along with an overall bandwidth and ETA indicator.
bool AcqTextStatus::Pulse(pkgAcquire* Owner) {
	pkgAcquireStatus::Pulse(Owner);

	rust::vec<Worker> list;
	for (pkgAcquire::Worker* I = Owner->WorkersBegin(); I != 0; I = Owner->WorkerStep(I)) {

		// There is no item running
		if (I->CurrentItem == 0) {
			list.push_back(Worker {
				false, I->Status, 0, "", "",
				#if RUST_APT_WORKER_SIZES == 1
				0, 0,
				#endif
				false,
			});
			continue;
		}

		list.push_back(Worker {
			true, I->Status, I->CurrentItem->Owner->ID,
			I->CurrentItem->ShortDesc, I->CurrentItem->Owner->ActiveSubprocess,
			#if RUST_APT_WORKER_SIZES == 1
			I->CurrentItem->CurrentSize, I->CurrentItem->TotalSize,
			#endif
			I->CurrentItem->Owner->Complete,
		});
	}

	pulse(callback, list, Percent, TotalBytes, CurrentBytes, CurrentCPS);
	Update = true;
	return true;
}


/// Not Yet Implemented
///
/// Invoked when the user should be prompted to change the inserted removable media.
bool AcqTextStatus::MediaChange(std::string Media, std::string Drive) {
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
}


/// Not Yet Implemented
///
/// Ask the user for confirmation of changes to infos about a repository
/// Must return true if the user accepts or false if not
bool AcqTextStatus::ReleaseInfoChanges(metaIndex const* const L,
metaIndex const* const N,
std::vector<ReleaseInfoChange>&& Changes) {
	(void)L;
	(void)N;
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

	// return YnPrompt(_("Do you want to accept these changes and continue updating from this repository?"), false, false, out, out);

	// Not yet implemented. Remove return true when it is.
	return true;
}

/// Calls for OpProgress usage.
OpProgressWrapper::OpProgressWrapper(DynOperationProgress& callback)
: callback(callback) {}

void OpProgressWrapper::Update() { op_update(callback, Op, Percent); }

void OpProgressWrapper::Done() { op_done(callback); }

/// Calls for InstallProgress usage.
PackageManagerWrapper::PackageManagerWrapper(DynInstallProgress& callback)
: callback(callback) {}

bool PackageManagerWrapper::StatusChanged(
std::string pkgname, unsigned int steps_done, unsigned int total_steps, std::string action) {
	inst_status_changed(callback, pkgname, steps_done, total_steps, action);
	return true;
}

void PackageManagerWrapper::Error(
std::string pkgname, unsigned int steps_done, unsigned int total_steps, std::string error) {
	inst_error(callback, pkgname, steps_done, total_steps, error);
}
