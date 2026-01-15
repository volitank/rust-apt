//! Contains miscellaneous helper utilities.
use std::cmp::Ordering;
use std::env;

use crate::error::AptErrors;
use crate::{Cache, DepFlags, Package, config};

fn env_usize_nonzero(key: &str) -> Option<usize> {
	env::var(key).ok()?.parse().ok().filter(|v: &usize| *v > 0)
}

fn ioctl_terminal_size() -> Option<(usize, usize)> {
	use std::ffi::{c_int, c_ulong};
	use std::io::{stderr, stdin, stdout};
	use std::mem::MaybeUninit;
	use std::os::unix::io::AsRawFd;

	#[repr(C)]
	struct Winsize {
		row: u16,
		col: u16,
		x_pixel: u16,
		y_pixel: u16,
	}

	extern "C" {
		fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
	}

	const TIOCGWINSZ: c_ulong = 0x5413;

	fn query(fd: c_int) -> Option<(usize, usize)> {
		let mut win_size = MaybeUninit::<Winsize>::uninit();
		let result = unsafe { ioctl(fd, TIOCGWINSZ, win_size.as_mut_ptr()) };
		if result != 0 {
			return None;
		}
		let win_size = unsafe { win_size.assume_init() };
		let cols = usize::from(win_size.col);
		let rows = usize::from(win_size.row);
		(cols != 0 && rows != 0).then_some((cols, rows))
	}

	query(stdout().as_raw_fd())
		.or_else(|| query(stderr().as_raw_fd()))
		.or_else(|| query(stdin().as_raw_fd()))
}

/// Get the terminal's height, i.e. the number of rows it has.
///
/// # Returns:
/// * The terminal height, or `24` if it cannot be determined.
pub fn terminal_height() -> usize {
	ioctl_terminal_size()
		.map(|(_, rows)| rows)
		.or_else(|| env_usize_nonzero("LINES"))
		.unwrap_or(24)
}

/// Get the terminal's width, i.e. the number of columns it has.
///
/// # Returns:
/// * The terminal width, or `80` if it cannot be determined.
pub fn terminal_width() -> usize {
	ioctl_terminal_size()
		.map(|(cols, _)| cols)
		.or_else(|| env_usize_nonzero("COLUMNS"))
		.unwrap_or(80)
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
	let result = raw::cmp_versions(ver1, ver2);
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
/// use rust_apt::new_cache;
/// use rust_apt::util::{unit_str, NumSys};
/// let cache = new_cache!().unwrap();
/// let pkg = cache.get("apt").unwrap();
/// let version = pkg.candidate().unwrap();
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
pub fn apt_lock() -> Result<(), AptErrors> {
	config::init_config_system();
	Ok(raw::apt_lock()?)
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
pub fn apt_lock_inner() -> Result<(), AptErrors> {
	config::init_config_system();
	Ok(raw::apt_lock_inner()?)
}

/// Unlock the Dpkg lockfile.
pub fn apt_unlock_inner() {
	config::init_config_system();
	raw::apt_unlock_inner()
}

/// Checks if any locks are currently active for the lockfile. Note that this
/// will only return [`true`] if the current process has an active lock, while
/// calls to [`apt_lock`] will return an [`AptErrors`] if another process has an
/// active lock.
pub fn apt_is_locked() -> bool {
	config::init_config_system();
	raw::apt_is_locked()
}

/// Reference implementation to print broken packages just like apt does.
///
/// ## Returns [`None`] if the package is not considered broken
///
/// ## now:
///   * [true] = When checking broken packages before modifying the cache.
///   * [false] = When checking broken packages after modifying the cache.
pub fn show_broken_pkg(cache: &Cache, pkg: &Package, now: bool) -> Option<String> {
	// If the package isn't broken for the state Return None
	if (now && !pkg.is_now_broken()) || (!now && !pkg.is_inst_broken()) {
		return None;
	};

	let mut broken_string = String::new();

	broken_string += &format!(" {pkg} :");

	// Pick the proper version based on now status.
	// else Return with just the package name like Apt does.
	let Some(ver) = (match now {
		true => pkg.installed(),
		false => pkg.install_version(),
	}) else {
		broken_string += "\n";
		return Some(broken_string);
	};

	let indent = pkg.name().len() + 3;
	let mut first = true;

	// ShowBrokenDeps
	for dep in ver.depends_map().values().flatten() {
		for (i, base_dep) in dep.iter().enumerate() {
			if !cache.depcache().is_important_dep(base_dep) {
				continue;
			}

			let dep_flag = if now { DepFlags::DepGNow } else { DepFlags::DepInstall };

			if cache.depcache().dep_state(base_dep) & dep_flag == dep_flag {
				continue;
			}

			if !first {
				broken_string += &" ".repeat(indent);
			}
			first = false;

			// If it's the first or Dep
			if i > 0 {
				broken_string += &" ".repeat(base_dep.dep_type().as_ref().len() + 3);
			} else {
				broken_string += &format!(" {}: ", base_dep.dep_type())
			}

			broken_string += base_dep.target_package().name();

			if let (Ok(ver_str), Some(comp)) = (base_dep.target_ver(), base_dep.comp_type()) {
				broken_string += &format!(" ({comp} {ver_str})");
			}

			let target = base_dep.target_package();
			if !target.has_provides() {
				if let Some(target_ver) = target.install_version() {
					broken_string += &format!(" but {target_ver} is to be installed")
				} else if target.candidate().is_some() {
					broken_string += " but it is not going to be installed";
				} else if target.has_provides() {
					broken_string += " but it is a virtual package";
				} else {
					broken_string += " but it is not installable";
				}
			}

			if i + 1 != dep.len() {
				broken_string += " or"
			}
			broken_string += "\n";
		}
	}
	Some(broken_string)
}

#[cxx::bridge]
pub(crate) mod raw {
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
		pub fn cmp_versions(ver1: &str, ver2: &str) -> i32;

		pub fn quote_string(string: &str, bad: String) -> String;

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
