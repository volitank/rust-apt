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
/// let ver1 = String::from("5.0");
/// let ver2 = String::from("6.0");
/// let result = cmp_versions(&ver1, &ver2);
///
/// assert_eq!(Ordering::Less, result);
/// ```
pub fn cmp_versions(ver1: &String, ver2: &String) -> Ordering {
	let result = apt::cmp_versions(ver1, ver2);

	if result < 0 {
		Ordering::Less
	} else if result == 0 {
		Ordering::Equal
	} else {
		Ordering::Greater
	}
}
