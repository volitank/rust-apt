//! Contains miscellaneous helper utilities.
use std::cmp::Ordering;

use crate::raw::apt;

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
	let result = apt::cmp_versions(ver1.to_owned(), ver2.to_owned());

	match apt::cmp_versions(ver1.to_owned(), ver2.to_owned()) {
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
