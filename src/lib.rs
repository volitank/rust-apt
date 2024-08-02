//! rust-apt provides bindings to `libapt-pkg`.
//! The goal is to eventually have all of the functionality of `python-apt`
//!
//! The source repository is <https://gitlab.com/volian/rust-apt>
//! For more information please see the readme in the source code.
//!
//! Each module has a `raw` submodule containing c++ bindings to `libapt-pkg`
//!
//! These are safe to use in terms of memory,
//! but may cause segfaults if you do something wrong.
//!
//! If you find a way to segfault without using the `libapt-pkg` bindings
//! directly, please report this as a bug.

// Clippy is really mad at my safety docs and idk why
#![allow(clippy::missing_safety_doc)]

#[macro_use]
mod macros;
mod acquire;
pub mod cache;
pub mod config;
mod depcache;
pub mod error;
mod iterators;
mod pkgmanager;
pub mod progress;
pub mod records;
pub mod tagfile;
pub mod util;

#[doc(inline)]
pub use cache::{Cache, PackageSort};
pub use iterators::dependency::{create_depends_map, BaseDep, DepFlags, DepType, Dependency};
pub use iterators::files::{PackageFile, VersionFile};
pub use iterators::package::{Marked, Package, PkgCurrentState, PkgInstState, PkgSelectedState};
pub use iterators::provider::Provider;
pub use iterators::version::Version;

/// C++ bindings for libapt-pkg
pub mod raw {
	pub use crate::acquire::raw::{
		acquire_status, create_acquire, AcqTextStatus, AcqWorker, Item, ItemDesc, ItemState,
		PkgAcquire,
	};
	pub use crate::cache::raw::{create_cache, PkgCacheFile};
	pub use crate::depcache::raw::{ActionGroup, PkgDepCache};
	pub use crate::iterators::{
		DepIterator, DescIterator, PkgFileIterator, PkgIterator, PrvIterator, VerFileIterator,
		VerIterator,
	};
	pub use crate::pkgmanager::raw::{
		create_pkgmanager, create_problem_resolver, PackageManager, ProblemResolver,
	};
	pub use crate::records::raw::{IndexFile, Parser, PkgRecords};
	pub use crate::util::raw::*;
	// Hmm, maybe this is reason enough to make a wrapper in C++
	// So that the raw functions are methods on a "Config" struct?
	// But it may need to not outlive the cache if we do that.
	pub mod config {
		pub use crate::config::raw::*;
	}

	/// Iterator trait for libapt raw bindings
	pub trait IntoRawIter {
		type Item;
		fn raw_iter(self) -> Self::Item;

		fn make_safe(self) -> Option<Self>
		where
			Self: Sized;

		fn to_vec(self) -> Vec<Self>
		where
			Self: Sized;
	}

	use cxx::UniquePtr;
	use paste::paste;

	raw_iter!(
		PkgIterator,
		VerIterator,
		DepIterator,
		PrvIterator,
		VerFileIterator,
		DescIterator,
		PkgFileIterator
	);
}

use depcache::DepCache;
use error::AptErrors;
use records::PackageRecords;

impl_deref!(
	Cache -> raw::PkgCacheFile,
	DepCache -> raw::PkgDepCache,
	PackageRecords -> raw::PkgRecords,
	AptErrors -> Vec<error::raw::AptError>,
	Package<'a> -> raw::PkgIterator,
	Version<'a> -> raw::VerIterator,
	Dependency<'a> -> Vec<BaseDep<'a>>,
	BaseDep<'a> -> raw::DepIterator,
	Provider<'a> -> raw::PrvIterator,
	VersionFile<'a> -> raw::VerFileIterator,
	PackageFile<'a> -> raw::PkgFileIterator,
);

// Version is omitted because it has special needs
impl_partial_eq!(
	Package<'a>,
	BaseDep<'a>,
	Provider<'a>,
	VersionFile<'a>,
	PackageFile<'a>,
);

impl_hash_eq!(
	Package<'a>,
	Version<'a>,
	BaseDep<'a>,
	Provider<'a>,
	VersionFile<'a>,
	PackageFile<'a>,
);
