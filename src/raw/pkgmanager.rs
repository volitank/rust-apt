//! Contains types and bindings for fetching and installing packages from the
//! cache.
use super::error::AptErrors;

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {
	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/progress.h");
		include!("rust-apt/apt-pkg-c/cache.h");
		include!("rust-apt/apt-pkg-c/records.h");
		include!("rust-apt/apt-pkg-c/util.h");
		include!("rust-apt/apt-pkg-c/pkgmanager.h");

		type PackageManager;
		type ProblemResolver;

		type Cache = crate::raw::cache::raw::Cache;
		type Package = crate::raw::cache::raw::Package;
		type Records = crate::raw::records::raw::Records;
		type DynAcquireProgress = crate::raw::progress::raw::DynAcquireProgress;
		type DynInstallProgress = crate::raw::progress::raw::DynInstallProgress;
		type DynOperationProgress = crate::raw::progress::raw::DynOperationProgress;

		pub fn create_pkgmanager(cache: &Cache) -> UniquePtr<PackageManager>;

		pub fn get_archives(
			self: &PackageManager,
			cache: &Cache,
			records: &Records,
			progress: &mut DynAcquireProgress,
		) -> Result<()>;

		pub fn do_install(self: &PackageManager, progress: &mut DynInstallProgress) -> Result<()>;

		pub fn create_problem_resolver(cache: &Cache) -> UniquePtr<ProblemResolver>;

		pub fn protect(self: &ProblemResolver, pkg: &Package);

		fn u_resolve(
			self: &ProblemResolver,
			fix_broken: bool,
			op_progress: &mut DynOperationProgress,
		) -> Result<()>;
	}
}

impl raw::ProblemResolver {
	pub fn resolve(
		&self,
		fix_broken: bool,
		op_progress: &mut raw::DynOperationProgress,
	) -> Result<(), AptErrors> {
		Ok(self.u_resolve(fix_broken, op_progress)?)
	}
}
