//! Contains types and bindings for fetching and installing packages from the
//! cache.
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use cxx::{Exception, UniquePtr};

use crate::cache::raw::PkgCacheFile;
use crate::progress::{AcquireProgress, InstallProgress};
use crate::records::Records;

pub(crate) struct PackageManager {
	ptr: Rc<RefCell<UniquePtr<raw::PkgPackageManager>>>,
	cache: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
}

// Other structs that use this one implement Debug, so we need to as well.
// TODO: Implement some actually useful information.
impl fmt::Debug for PackageManager {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "PackageManager {{ NO DEBUG IMPLEMENTED YET }}")
	}
}

impl PackageManager {
	pub fn new(cache: Rc<RefCell<UniquePtr<PkgCacheFile>>>) -> Self {
		let ptr = Rc::new(RefCell::new(raw::pkgmanager_create(&cache.borrow())));
		Self { ptr, cache }
	}

	pub fn get_archives(
		&self,
		records: &mut Records,
		progress: &mut Box<dyn AcquireProgress>,
	) -> Result<(), Exception> {
		raw::pkgmanager_get_archives(
			&self.ptr.borrow(),
			&self.cache.borrow(),
			&mut records.ptr,
			progress,
		)
	}

	pub fn do_install(&self, progress: &mut Box<dyn InstallProgress>) -> Result<(), Exception> {
		raw::pkgmanager_do_install(&self.ptr.borrow(), progress)
	}
}

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {
	unsafe extern "C++" {
		type PkgPackageManager;

		type PkgCacheFile = crate::cache::raw::PkgCacheFile;
		type DynAcquireProgress = crate::progress::raw::DynAcquireProgress;
		type DynInstallProgress = crate::progress::raw::DynInstallProgress;
		type Records = crate::records::raw::Records;

		include!("rust-apt/apt-pkg-c/pkgmanager.h");

		pub fn pkgmanager_create(cache: &UniquePtr<PkgCacheFile>) -> UniquePtr<PkgPackageManager>;

		pub fn pkgmanager_get_archives(
			pkgmanager: &UniquePtr<PkgPackageManager>,
			cache: &UniquePtr<PkgCacheFile>,
			records: &mut Records,
			progress: &mut DynAcquireProgress,
		) -> Result<()>;

		pub fn pkgmanager_do_install(
			pkgmanager: &UniquePtr<PkgPackageManager>,
			progress: &mut DynInstallProgress,
		) -> Result<()>;
	}
}
