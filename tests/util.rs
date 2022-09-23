mod util {
	use std::cmp::Ordering;

	use rust_apt::util;

	#[test]
	fn cmp_versions() {
		let ver1 = "5.0";
		let ver2 = "6.0";

		assert_eq!(Ordering::Less, util::cmp_versions(ver1, ver2));
		assert_eq!(Ordering::Equal, util::cmp_versions(ver1, ver1));
		assert_eq!(Ordering::Greater, util::cmp_versions(ver2, ver1));
	}
}
