//! Contains types and bindings for fetching and installing packages from the
//! cache.

#[cxx::bridge]
pub(crate) mod raw {
	#[repr(u32)]
	enum OrderResult {
		Completed,
		Failed,
		Incomplete,
	}

	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/pkgmanager.h");

		type PackageManager;
		type ProblemResolver;
		type OrderResult;

		type PkgCacheFile = crate::cache::raw::PkgCacheFile;
		type PkgIterator = crate::cache::raw::PkgIterator;
		type PkgRecords = crate::records::raw::PkgRecords;
		type PkgDepCache = crate::depcache::raw::PkgDepCache;
		type AcqTextStatus = crate::acquire::raw::AcqTextStatus;

		type InstallProgressFancy<'a> = crate::progress::InstallProgressFancy<'a>;
		type OperationProgress<'a> = crate::progress::OperationProgress<'a>;

		/// # Safety
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn create_pkgmanager(depcache: &PkgDepCache) -> UniquePtr<PackageManager>;

		pub fn get_archives(
			self: &PackageManager,
			cache: &PkgCacheFile,
			records: &PkgRecords,
			progress: Pin<&mut AcqTextStatus>,
		) -> Result<()>;

		pub fn do_install(
			self: &PackageManager,
			progress: Pin<&mut InstallProgressFancy>,
		) -> OrderResult;

		/// Send dpkg status messages to an File Descriptor.
		/// This required more work to implement but is the most flexible.
		pub fn do_install_fd(self: &PackageManager, fd: i32) -> OrderResult;

		/// # Safety
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn create_problem_resolver(depcache: &PkgDepCache) -> UniquePtr<ProblemResolver>;

		pub fn protect(self: &ProblemResolver, pkg: &PkgIterator);

		fn resolve(
			self: &ProblemResolver,
			fix_broken: bool,
			op_progress: Pin<&mut OperationProgress>,
		) -> Result<()>;
	}
}
