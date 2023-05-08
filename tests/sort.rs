mod sort {
	use rust_apt::cache::*;
	use rust_apt::new_cache;

	#[test]
	fn defaults() {
		let cache = new_cache!().unwrap();
		let mut installed = false;
		let mut auto_installed = false;

		// Test defaults and ensure there are no virtual packages.
		// And that we have any packages at all.
		let mut real_pkgs = Vec::new();
		let mut virtual_pkgs = Vec::new();

		let sort = PackageSort::default();

		for pkg in cache.packages(&sort).unwrap() {
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
		let cache = new_cache!().unwrap();

		// Check that we have virtual and real packages after sorting.
		let mut real_pkgs = Vec::new();
		let mut virtual_pkgs = Vec::new();

		let sort = PackageSort::default().include_virtual().names();

		for pkg in cache.packages(&sort).unwrap() {
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
		let cache = new_cache!().unwrap();

		// Check that we have only virtual packages.
		let mut real_pkgs = Vec::new();
		let mut virtual_pkgs = Vec::new();

		let sort = PackageSort::default().only_virtual();

		for pkg in cache.packages(&sort).unwrap() {
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
		let cache = new_cache!().unwrap();

		let sort = PackageSort::default().upgradable();
		for pkg in cache.packages(&sort).unwrap() {
			assert!(pkg.is_upgradable())
		}

		let sort = PackageSort::default().not_upgradable();
		for pkg in cache.packages(&sort).unwrap() {
			assert!(!pkg.is_upgradable())
		}
	}

	#[test]
	fn installed() {
		let cache = new_cache!().unwrap();

		let sort = PackageSort::default().installed();
		for pkg in cache.packages(&sort).unwrap() {
			assert!(pkg.is_installed())
		}

		let sort = PackageSort::default().not_installed();
		for pkg in cache.packages(&sort).unwrap() {
			assert!(!pkg.is_installed())
		}
	}

	#[test]
	fn auto_installed() {
		let cache = new_cache!().unwrap();

		let sort = PackageSort::default().auto_installed();
		for pkg in cache.packages(&sort).unwrap() {
			println!("{}", pkg.name());
			assert!(pkg.is_auto_installed())
		}

		let sort = PackageSort::default().manually_installed();
		for pkg in cache.packages(&sort).unwrap() {
			assert!(!pkg.is_auto_installed());
		}
	}

	#[test]
	fn auto_removable() {
		let cache = new_cache!().unwrap();

		let sort = PackageSort::default().auto_removable();
		for pkg in cache.packages(&sort).unwrap() {
			assert!(pkg.is_auto_removable())
		}

		let sort = PackageSort::default().not_auto_removable();
		for pkg in cache.packages(&sort).unwrap() {
			assert!(!pkg.is_auto_removable())
		}
	}
}
