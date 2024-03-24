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
