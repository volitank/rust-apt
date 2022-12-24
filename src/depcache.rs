use std::ops::Deref;

use cxx::Exception;

use crate::raw::depcache::raw;
use crate::raw::progress::NoOpProgress;
use crate::util::DiskSpace;

type RawDepCache = raw::DepCache;

pub struct DepCache {
	ptr: RawDepCache,
}

impl DepCache {
	pub fn new(ptr: RawDepCache) -> DepCache { DepCache { ptr } }

	/// Clear any marked changes in the DepCache.
	pub fn clear_marked(&self) -> Result<(), Exception> {
		// Use our dummy OperationProgress struct.
		self.init(&mut NoOpProgress::new_box())
	}

	/// The amount of space required for installing/removing the packages,"
	///
	/// i.e. the Installed-Size of all packages marked for installation"
	/// minus the Installed-Size of all packages for removal."
	pub fn disk_size(&self) -> DiskSpace {
		let size = self.ptr.disk_size();
		if size < 0 {
			return DiskSpace::Free(-size as u64);
		}
		DiskSpace::Require(size as u64)
	}
}

impl Deref for DepCache {
	type Target = RawDepCache;

	#[inline]
	fn deref(&self) -> &RawDepCache { &self.ptr }
}
