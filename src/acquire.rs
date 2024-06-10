//! File Acquiration
//!
//! The following was taken from libapt-pkg documentation.
//!
//! This module contains the Acquire system. It is responsible for bringing
//! files into the local pathname space. It deals with URIs for files and
//! URI handlers responsible for downloading or finding the URIs.
//!
//! Each file to download is represented by an Acquire::Item class subclassed
//! into a specialization. The Item class can add itself to several URI
//! acquire queues each prioritized by the download scheduler. When the
//! system is run the proper URI handlers are spawned and the acquire
//! queues are fed into the handlers by the schedular until the queues are
//! empty. This allows for an Item to be downloaded from an alternate source
//! if the first try turns out to fail. It also allows concurrent downloading
//! of multiple items from multiple sources as well as dynamic balancing
//! of load between the sources.
//!
//! Scheduling of downloads is done on a first ask first get basis. This
//! preserves the order of the download as much as possible. And means the
//! fastest source will tend to process the largest number of files.
//!
//! Internal methods and queues for performing gzip decompression,
//! md5sum hashing and file copying are provided to allow items to apply
//! a number of transformations to the data files they are working with.

#[cxx::bridge]
pub(crate) mod raw {
	#[repr(u32)]
	enum ItemState {
		StatIdle,
		StatFetching,
		StatDone,
		StatError,
		StatAuthError,
		StatTransientNetworkError,
	}

	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/acquire.h");
		type AcqTextStatus;
		type PkgAcquire;
		type AcqWorker;
		type ItemDesc;
		type Item;
		type ItemState;

		type AcquireProgress<'a> = crate::progress::AcquireProgress<'a>;

		/// A client-supplied unique identifier.
		///
		/// APT progress reporting will store an ID as shown in "Get:42…".
		pub fn id(self: &Item) -> u32;
		/// `true`` if entire object has been successfully fetched.
		pub fn complete(self: &Item) -> bool;
		/// The size of the object to fetch.
		pub fn file_size(self: &Item) -> u64;
		/// Get the URI of the item.
		pub fn uri(self: &Item) -> String;
		/// The Destination file path.
		pub fn dest_file(self: &Item) -> String;
		/// The current status of this item.
		pub fn status(self: &Item) -> ItemState;
		/// Contains a textual description of the error encountered
		/// if ItemState is StatError or StatAuthError.
		pub fn error_text(self: &Item) -> String;
		/// Contains the name of the subprocess that is operating on this item.
		///
		/// For instance, "store", "gzip", "rred", "gpgv".
		pub fn active_subprocess(self: &Item) -> String;
		/// The acquire process with which this item is associated.
		pub fn owner(self: &Item) -> UniquePtr<PkgAcquire>;

		/// URI from which to download this item.
		pub fn uri(self: &ItemDesc) -> String;
		/// Description of this item.
		pub fn description(self: &ItemDesc) -> String;
		/// Shorter description of this item.
		pub fn short_desc(self: &ItemDesc) -> String;
		/// Underlying item which is to be downloaded.
		pub fn owner(self: &ItemDesc) -> UniquePtr<Item>;

		/// # Safety
		///
		/// Before you do anything with AcqTextStatus you must set the callback.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn acquire_status() -> UniquePtr<AcqTextStatus>;
		/// # Safety
		///
		/// Setting the Callback requires a raw mutable pointer to
		/// AcquireProgress.
		///
		/// AcquireProgress must not be moved in memory or will segfault when
		/// AcqTextStatus calls it.
		unsafe fn set_callback(self: Pin<&mut AcqTextStatus>, progress: *mut AcquireProgress);

		/// The number of bytes fetched as of the most recent call
		/// to pkgAcquireStatus::Pulse, including local items.
		pub fn current_cps(self: &AcqTextStatus) -> u64;
		/// The amount of time that has elapsed since the download started.
		pub fn elapsed_time(self: &AcqTextStatus) -> u64;
		/// The total number of bytes accounted for by items that were
		/// successfully fetched.
		pub fn fetched_bytes(self: &AcqTextStatus) -> u64;
		/// The number of bytes fetched as of the most recent call to
		/// pkgAcquireStatus::Pulse, including local items.
		pub fn current_bytes(self: &AcqTextStatus) -> u64;
		/// The total number of bytes that need to be fetched.
		///
		/// This member is inaccurate, as new items might be enqueued while the
		/// download is in progress!
		pub fn total_bytes(self: &AcqTextStatus) -> u64;
		/// The estimated percentage of the download (0-100)
		pub fn percent(self: &AcqTextStatus) -> f64;

		/// The most recent status string received from the subprocess.
		pub fn status(self: &AcqWorker) -> String;
		/// The queue entry that is currently being downloaded.
		pub fn item(self: &AcqWorker) -> Result<UniquePtr<ItemDesc>>;
		/// How many bytes of the file have been downloaded.
		///
		/// Zero if the current progress of the file cannot be determined.
		pub fn current_size(self: &AcqWorker) -> u64;
		/// The total number of bytes to be downloaded.
		///
		/// Zero if the total size of the final is unknown.
		pub fn total_size(self: &AcqWorker) -> u64;

		// TODO: This should probably be unsafe, but not sure at the moment how to
		// handle it I guess we would need to wrap PkgAcquire and AcqWorker so they can
		// have proper lifetimes?

		/// CxxVector of active workers
		pub fn workers(self: &PkgAcquire) -> UniquePtr<CxxVector<AcqWorker>>;

		/// Get the ItemDesc that contain the source list URIs
		///
		/// # Safety
		///
		/// You must not let these out of scope of PkgAcquire. SIGABRT.
		unsafe fn uris(self: &PkgAcquire) -> UniquePtr<CxxVector<ItemDesc>>;

		// It isn't clear that create_acquire should be unsafe.
		// It doesn't segfault if you drop the Cache.
		// But it does return a UniquePtr so I assume it is unsafe.

		/// Create PkgAcquire.
		///
		/// # Safety
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn create_acquire() -> UniquePtr<PkgAcquire>;
	}
}
