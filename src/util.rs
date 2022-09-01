//! Contains miscellaneous helper utilities.
use std::cmp::Ordering;

pub use cxx::Exception;

use crate::config;

/// Get the terminal's height, i.e. the number of rows it has.
///
/// # Returns:
/// * The terminal height, or `24` if it cannot be determined.
pub fn terminal_height() -> usize {
	if let Some(size) = termsize::get() {
		usize::from(size.rows)
	} else {
		24
	}
}

/// Get the terminal's width, i.e. the number of columns it has.
///
/// # Returns:
/// * The terminal width, or `80` if it cannot be determined.
pub fn terminal_width() -> usize {
	if let Some(size) = termsize::get() {
		usize::from(size.cols)
	} else {
		80
	}
}

/// Compares two package versions, `ver1` and `ver2`. The returned enum variant
/// applies to the first version passed in.
///
/// # Examples
/// ```
/// use rust_apt::util::cmp_versions;
/// use std::cmp::Ordering;
///
/// let ver1 = "5.0";
/// let ver2 = "6.0";
/// let result = cmp_versions(ver1, ver2);
///
/// assert_eq!(Ordering::Less, result);
/// ```
pub fn cmp_versions(ver1: &str, ver2: &str) -> Ordering {
	let result = raw::cmp_versions(ver1.to_owned(), ver2.to_owned());
	match result {
		_ if result < 0 => Ordering::Less,
		_ if result == 0 => Ordering::Equal,
		_ => Ordering::Greater,
	}
}

/// Disk Space that `apt` will use for a transaction.
pub enum DiskSpace {
	/// Additional Disk Space required.
	Require(u64),
	/// Disk Space that will be freed
	Free(u64),
}

/// Numeral System for unit conversion.
pub enum NumSys {
	/// Base 2 | 1024 | KibiByte (KiB)
	Binary,
	/// Base 10 | 1000 | KiloByte (KB)
	Decimal,
}

/// Converts bytes into human readable output.
///
/// ```
/// use rust_apt::cache::Cache;
/// use rust_apt::util::{unit_str, NumSys};
/// let cache = Cache::new();
/// let version = cache.get("apt").unwrap().candidate().unwrap();
///
/// println!("{}", unit_str(version.size(), NumSys::Decimal));
/// ```
pub fn unit_str(val: u64, base: NumSys) -> String {
	let val = val as f64;
	let (num, tera, giga, mega, kilo) = match base {
		NumSys::Binary => (1024.0_f64, "TiB", "GiB", "MiB", "KiB"),
		NumSys::Decimal => (1000.0_f64, "TB", "GB", "MB", "KB"),
	};

	let powers = [
		(num.powi(4), tera),
		(num.powi(3), giga),
		(num.powi(2), mega),
		(num, kilo),
	];

	for (divisor, unit) in powers {
		if val > divisor {
			return format!("{:.2} {unit}", val / divisor);
		}
	}
	format!("{val} B")
}

/// Converts seconds into a human readable time string.
pub fn time_str(seconds: u64) -> String {
	if seconds > 60 * 60 * 24 {
		return format!(
			"{}d {}h {}min {}s",
			seconds / 60 / 60 / 24,
			(seconds / 60 / 60) % 24,
			(seconds / 60) % 60,
			seconds % 60,
		);
	}
	if seconds > 60 * 60 {
		return format!(
			"{}h {}min {}s",
			(seconds / 60 / 60) % 24,
			(seconds / 60) % 60,
			seconds % 60,
		);
	}
	if seconds > 60 {
		return format!("{}min {}s", (seconds / 60) % 60, seconds % 60,);
	}
	format!("{seconds}s")
}

/// Get an APT-styled progress bar.
///
/// # Returns:
/// * [`String`] representing the progress bar.
///
/// # Example:
/// ```
/// use rust_apt::util::get_apt_progress_string;
/// let progress = get_apt_progress_string(0.5, 10);
/// assert_eq!(progress, "[####....]");
/// ```
pub fn get_apt_progress_string(percent: f32, output_width: u32) -> String {
	raw::get_apt_progress_string(percent, output_width)
}

/// Lock the APT lockfile.
/// This should be done before modifying any APT files
/// such as with [`crate::cache::Cache::update`]
/// and then [`apt_unlock`] should be called after.
///
/// This Function Requires root
///
/// If [`apt_lock`] is called `n` times, [`apt_unlock`] must also be called `n`
/// times to release all acquired locks.
///
/// # Known Error Messages:
/// * `E:Could not open lock file /var/lib/dpkg/lock-frontend - open (13:
///   Permission denied)`
/// * `E:Unable to acquire the dpkg frontend lock (/var/lib/dpkg/lock-frontend),
///   are you root?`
pub fn apt_lock() -> Result<(), Exception> {
	config::init_config_system();
	raw::apt_lock()
}

/// Unlock the APT lockfile.
pub fn apt_unlock() {
	config::init_config_system();
	raw::apt_unlock()
}

/// Unlock the Dpkg lockfile.
/// This should be done before manually running
/// [`crate::cache::Cache::do_install`]
/// and then [`apt_unlock_inner`] should be called after.
///
/// This Function Requires root
pub fn apt_lock_inner() -> Result<(), Exception> {
	config::init_config_system();
	raw::apt_lock_inner()
}

/// Unlock the Dpkg lockfile.
pub fn apt_unlock_inner() {
	config::init_config_system();
	raw::apt_unlock_inner()
}

/// Checks if any locks are currently active for the lockfile. Note that this
/// will only return [`true`] if the current process has an active lock, calls
/// to [`apt_lock`] will return an [`Exception`] if another process has an
/// active lock.
pub fn apt_is_locked() -> bool {
	config::init_config_system();
	raw::apt_is_locked()
}

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {

	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/util.h");

		/// Compares two package versions, `ver1` and `ver2`. The returned
		/// integer's value is mapped to one of the following integers:
		/// - Less than 0: `ver1` is less than `ver2`.
		/// - Equal to 0: `ver1` is equal to `ver2`.
		/// - Greater than 0: `ver1` is greater than `ver2`.
		///
		/// Unless you have a specific need for otherwise, you should probably
		/// use [`crate::util::cmp_versions`] instead.
		pub fn cmp_versions(ver1: String, ver2: String) -> i32;

		/// Return an APT-styled progress bar (`[####..]`).
		pub fn get_apt_progress_string(percent: f32, output_width: u32) -> String;

		/// Lock the lockfile.
		// TODO: There's `unlock_inner` functions in the Python APT library, but I have no clue how
		// we'd implement them in regard to our structs and such in this library. They seem to only
		// be used to have a lock on dpkg between calls, which shouldn't be an issue in most cases,
		// though it should probably be looked into.
		pub fn apt_lock() -> Result<()>;

		/// Unock the lockfile.
		pub fn apt_unlock();

		/// Lock the Dpkg lockfile.
		pub fn apt_lock_inner() -> Result<()>;

		/// Unlock the Dpkg lockfile.
		pub fn apt_unlock_inner();

		/// Check if the lockfile is locked.
		pub fn apt_is_locked() -> bool;
	}
}
