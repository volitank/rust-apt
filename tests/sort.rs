mod sort {
	use rust_apt::cache::*;

	#[test]
	fn defaults() {
		let cache = Cache::new();
		let mut installed = false;
		let mut auto_installed = false;

		// Test defaults and ensure there are no virtual packages.
		// And that we have any packages at all.
		let mut real_pkgs = Vec::new();
		let mut virtual_pkgs = Vec::new();

		let sort = PackageSort::default();

		for pkg in cache.packages(&sort) {
			if pkg.is_auto_installed() {
				auto_installed = true;
			}
			if pkg.is_installed() {
				installed = true;
			}

			if pkg.has_versions() {
				real_pkgs.push(pkg);
				continue;
			}
			virtual_pkgs.push(pkg);
		}
		assert!(!real_pkgs.is_empty());
		assert!(virtual_pkgs.is_empty());
		assert!(auto_installed);
		assert!(installed)
	}

	#[test]
	fn include_virtual() {
		let cache = Cache::new();

		// Check that we have virtual and real packages after sorting.
		let mut real_pkgs = Vec::new();
		let mut virtual_pkgs = Vec::new();

		let sort = PackageSort::default().include_virtual();

		for pkg in cache.packages(&sort) {
			if pkg.has_versions() {
				real_pkgs.push(pkg);
				continue;
			}
			virtual_pkgs.push(pkg);
		}
		assert!(!real_pkgs.is_empty());
		assert!(!virtual_pkgs.is_empty());
	}

	#[test]
	fn only_virtual() {
		let cache = Cache::new();

		// Check that we have only virtual packages.
		let mut real_pkgs = Vec::new();
		let mut virtual_pkgs = Vec::new();

		let sort = PackageSort::default().only_virtual();

		for pkg in cache.packages(&sort) {
			if pkg.has_versions() {
				real_pkgs.push(pkg);
				continue;
			}
			virtual_pkgs.push(pkg);
		}
		assert!(real_pkgs.is_empty());
		assert!(!virtual_pkgs.is_empty());
	}

	#[test]
	fn upgradable() {
		let cache = Cache::new();

		let sort = PackageSort::default().upgradable();
		for pkg in cache.packages(&sort) {
			// Sorting by upgradable skips the pkgDepCache same as `.is_upgradable(true)`
			// Here we check is_upgradable with the pkgDepCache to make sure there is
			// consistency
			assert!(pkg.is_upgradable(false))
		}

		let sort = PackageSort::default().not_upgradable();
		for pkg in cache.packages(&sort) {
			assert!(!pkg.is_upgradable(false))
		}
	}

	#[test]
	fn installed() {
		let cache = Cache::new();

		let sort = PackageSort::default().installed();
		for pkg in cache.packages(&sort) {
			assert!(pkg.is_installed())
		}

		let sort = PackageSort::default().not_installed();
		for pkg in cache.packages(&sort) {
			assert!(!pkg.is_installed())
		}
	}

	#[test]
	fn auto_installed() {
		let cache = Cache::new();

		let sort = PackageSort::default().auto_installed();
		for pkg in cache.packages(&sort) {
			assert!(pkg.is_auto_installed())
		}

		let sort = PackageSort::default().manually_installed();
		for pkg in cache.packages(&sort) {
			assert!(!pkg.is_auto_installed());
		}
	}

	#[test]
	fn auto_removable() {
		let cache = Cache::new();

		let sort = PackageSort::default().auto_removable();
		for pkg in cache.packages(&sort) {
			assert!(pkg.is_auto_removable())
		}

		let sort = PackageSort::default().not_auto_removable();
		for pkg in cache.packages(&sort) {
			assert!(!pkg.is_auto_removable())
		}
	}
}
