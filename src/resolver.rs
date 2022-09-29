//! Contains dependency resolution related structs.
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use cxx::UniquePtr;
use once_cell::unsync::OnceCell;

use crate::cache::raw::{PackagePtr, PkgCacheFile};
use crate::progress::OperationProgress;
use crate::util::Exception;

/// Internal struct for managing a pkgProblemResolver.
pub(crate) struct ProblemResolver {
	ptr: OnceCell<UniquePtr<raw::PkgProblemResolver>>,
	cache: Rc<RefCell<UniquePtr<PkgCacheFile>>>,
}

// Other structs that use this one implement Debug, so we need to as well.
// TODO: Implement some actually useful information.
impl fmt::Debug for ProblemResolver {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "ProblemResolver {{ NO DEBUG IMPLEMENTED YET }}")
	}
}

impl ProblemResolver {
	pub(crate) fn new(cache: Rc<RefCell<UniquePtr<PkgCacheFile>>>) -> Self {
		Self {
			ptr: OnceCell::new(),
			cache,
		}
	}

	// Internal method for lazily initializing the DepCache
	fn get_ptr(&self) -> &UniquePtr<raw::PkgProblemResolver> {
		println!("TEST");
		self.ptr
			.get_or_init(|| raw::problem_resolver_create(&self.cache.borrow()))
	}

	pub fn protect(&self, pkg_ptr: &PackagePtr) { raw::resolver_protect(self.get_ptr(), pkg_ptr); }

	pub fn resolve(
		&self,
		fix_broken: bool,
		op_progress: &mut Box<dyn OperationProgress>,
	) -> Result<(), Exception> {
		raw::resolver_resolve(self.get_ptr(), fix_broken, op_progress)
	}
}

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {
	unsafe extern "C++" {
		type PkgProblemResolver;
		type PkgCacheFile = crate::cache::raw::PkgCacheFile;
		type PackagePtr = crate::cache::raw::PackagePtr;
		type DynOperationProgress = crate::progress::raw::DynOperationProgress;

		include!("rust-apt/apt-pkg-c/cache.h");
		include!("rust-apt/apt-pkg-c/resolver.h");
		include!("rust-apt/apt-pkg-c/progress.h");

		pub fn problem_resolver_create(
			cache: &UniquePtr<PkgCacheFile>,
		) -> UniquePtr<PkgProblemResolver>;
		pub fn resolver_protect(resolver: &UniquePtr<PkgProblemResolver>, pkg: &PackagePtr);

		// TODO: What kind of errors can be returned here?
		// Research and update higher level structs as well
		// TODO: Create custom errors when we have better information
		pub fn resolver_resolve(
			resolver: &UniquePtr<PkgProblemResolver>,
			fix_broken: bool,
			op_progress: &mut DynOperationProgress,
		) -> Result<()>;
	}
}
