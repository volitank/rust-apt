//! Contains miscellaneous helper utilities.
use std::cmp::Ordering;

use terminal_size::{terminal_size, Height, Width};

use crate::cache::Cache;
use crate::config;
use crate::package::Package;
use crate::raw::error::AptErrors;
use crate::raw::package::DepFlags;
use crate::raw::util::raw;

/// Get the terminal's height, i.e. the number of rows it has.
///
/// # Returns:
/// * The terminal height, or `24` if it cannot be determined.
pub fn terminal_height() -> usize {
	if let Some((_, Height(rows))) = terminal_size() {
		usize::from(rows)
	} else {
		24
	}
}

/// Get the terminal's width, i.e. the number of columns it has.
///
/// # Returns:
/// * The terminal width, or `80` if it cannot be determined.
pub fn terminal_width() -> usize {
	if let Some((Width(cols), _)) = terminal_size() {
		usize::from(cols)
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
/// will only return [`true`] if the current process has an active lock, calls
/// to [`apt_lock`] will return an [`Exception`] if another process has an
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
		false => cache.depcache().install_version(pkg),
	}) else {
		broken_string += "\n";
		return Some(broken_string);
	};

	let indent = pkg.name().len() + 3;
	let mut first = true;

	// ShowBrokenDeps
	for dep in ver.depends_map().values().flatten() {
		for (i, base_dep) in dep.base_deps.iter().enumerate() {
			if !cache.depcache().is_important_dep(base_dep) {
				continue;
			}

			let dep_flag = if now { DepFlags::DepGnow } else { DepFlags::DepInstall };

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

			if let (Ok(ver_str), Some(comp)) = (base_dep.target_ver(), base_dep.comp()) {
				broken_string += &format!(" ({comp} {ver_str})");
			}

			let target = base_dep.target_package();
			if !target.has_provides() {
				if let Some(target_ver) = cache.depcache().install_version(target) {
					broken_string += &format!(" but {target_ver} is to be installed")
				} else if target.candidate().is_some() {
					broken_string += " but it is not going to be installed";
				} else if target.has_provides() {
					broken_string += " but it is a virtual package";
				} else {
					broken_string += " but it is not installable";
				}
			}

			if i + 1 != dep.base_deps.len() {
				broken_string += " or"
			}
			broken_string += "\n";
		}
	}
	Some(broken_string)
}
