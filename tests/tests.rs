#[cfg(test)]
mod cache {
	use rust_apt::cache;
	use rust_apt::cache::*;

	#[test]
	fn version_vec() {
		let cache = Cache::new();

		let mut versions = Vec::new();
		// Don't unwrap and assign so that the package can
		// Get out of scope from the version
		if let Some(apt) = cache.get("apt") {
			for version in apt.versions() {
				versions.push(version);
			}
		}
		// Apt is now out of scope
		assert!(!versions.is_empty());
	}

	#[test]
	fn all_packages() {
		let cache = Cache::new();
		let sort = PackageSort::default();

		// All real packages should not be empty.
		assert!(!cache.packages(&sort).collect::<Vec<_>>().is_empty());
		for pkg in cache.packages(&sort) {
			println!("{pkg}")
		}
	}

	#[test]
	fn descriptions() {
		let cache = Cache::new();

		// Apt should exist
		let pkg = cache.get("apt").unwrap();
		// Apt should have a candidate
		let cand = pkg.candidate().unwrap();
		// Apt should be installed
		let inst = pkg.installed().unwrap();

		// Assign installed descriptions
		let inst_sum = inst.summary();
		let inst_desc = inst.description();

		// Assign candidate descriptions
		let cand_sum = cand.summary();
		let cand_desc = cand.description();

		// If the lookup fails for whatever reason
		// The summary and description are the same
		assert_ne!(inst_sum, inst_desc);
		assert_ne!(cand_sum, cand_desc);
	}

	#[test]
	fn version_uris() {
		let cache = Cache::new();
		let pkg = cache.get("apt").unwrap();
		// Only test the candidate.
		// It's possible for the installed version to have no uris
		let cand = pkg.candidate().unwrap();
		assert!(!cand.uris().collect::<Vec<_>>().is_empty());
	}

	#[test]
	fn depcache_marked() {
		let cache = Cache::new();
		let pkg = cache.get("apt").unwrap();
		assert!(!pkg.marked_install());
		assert!(!pkg.marked_upgrade());
		assert!(!pkg.marked_delete());
		assert!(pkg.marked_keep());
		assert!(!pkg.marked_downgrade());
		assert!(!pkg.marked_reinstall());
		assert!(!pkg.is_now_broken());
		assert!(!pkg.is_inst_broken());
	}

	#[test]
	fn hashes() {
		let cache = Cache::new();
		let pkg = cache.get("apt").unwrap();
		let version_list = pkg.versions().collect::<Vec<_>>();
		// Should not be zero for apt
		assert!(!version_list.is_empty());
		for version in pkg.versions() {
			assert!(version.sha256().is_some());
			assert!(version.hash("sha256").is_some());
			assert!(version.sha512().is_none());
			assert!(version.hash("md5sum").is_some());
			assert!(version.hash("sha1").is_none())
		}
	}

	#[test]
	fn provides() {
		let cache = Cache::new();
		if let Some(pkg) = cache.get("www-browser") {
			let provides = cache.provides(&pkg, true).collect::<Vec<_>>();
			assert!(!provides.is_empty());
		};
	}

	#[test]
	fn depends() {
		let cache = Cache::new();

		let pkg = cache.get("apt").unwrap();
		let cand = pkg.candidate().unwrap();
		// Apt candidate should have dependencies
		for deps in cand.dependencies().unwrap() {
			for dep in &deps.base_deps {
				// Apt Dependencies should have targets
				assert!(!dep.all_targets().collect::<Vec<_>>().is_empty());
			}
		}
		assert!(cand.recommends().is_some());
		assert!(cand.suggests().is_some());
		// TODO: Add these as methods
		assert!(cand.get_depends("Replaces").is_some());
		assert!(cand.get_depends("Conflicts").is_some());
		assert!(cand.get_depends("Breaks").is_some());

		// This part is basically just formatting an apt depends String
		// Like you would see in `apt show`
		let mut dep_str = String::new();
		dep_str.push_str("Depends: ");
		for dep in cand.depends_map().get("Depends").unwrap() {
			if dep.is_or() {
				let mut or_str = String::new();
				let total = dep.base_deps.len() - 1;
				for (num, base_dep) in dep.base_deps.iter().enumerate() {
					or_str.push_str(base_dep.name());
					if !base_dep.comp().is_empty() {
						or_str.push_str(&format!("({} {})", base_dep.comp(), base_dep.version()))
					}
					if num != total {
						or_str.push_str(" | ");
					} else {
						or_str.push_str(", ");
					}
				}
				dep_str.push_str(&or_str)
			} else {
				let lone_dep = dep.first();
				dep_str.push_str(lone_dep.name().as_str());
				if !lone_dep.comp().is_empty() {
					dep_str.push_str(&format!(" ({} {})", lone_dep.comp(), lone_dep.version()))
				}
				dep_str.push_str(", ");
			}
		}
		println!("{dep_str}");
	}

	#[test]
	fn sources() {
		let cache = Cache::new();
		// If the source lists don't exists there is problems.
		assert!(!cache.sources().collect::<Vec<_>>().is_empty());
	}

	#[test]
	fn cache_count() {
		let cache = Cache::new();
		match cache.disk_size() {
			DiskSpace::Require(num) => {
				assert_eq!(num, 0);
			},
			DiskSpace::Free(num) => {
				panic!("Whoops it should be 0, not {num}.");
			},
		}
	}

	#[test]
	fn unit_str() {
		let testcase = [
			(1649267441664_u64, "1.50 TiB", "1.65 TB"),
			(1610612736_u64, "1.50 GiB", "1.61 GB"),
			(1572864_u64, "1.50 MiB", "1.57 MB"),
			(1536_u64, "1.50 KiB", "1.54 KB"),
			(1024_u64, "1024 B", "1.02 KB"),
			(1_u64, "1 B", "1 B"),
		];

		for (num, binary, decimal) in testcase {
			assert_eq!(binary, cache::unit_str(num, NumSys::Binary));
			assert_eq!(decimal, cache::unit_str(num, NumSys::Decimal));
		}
	}
}

#[cfg(test)]
mod sort {
	use rust_apt::cache::*;

	#[test]
	fn defaults() {
		let cache = Cache::new();

		// Test defaults and ensure there are no virtual packages.
		// And that we have any packages at all.
		let mut real_pkgs = Vec::new();
		let mut virtual_pkgs = Vec::new();

		let sort = PackageSort::default();

		for pkg in cache.packages(&sort) {
			if pkg.has_versions() {
				real_pkgs.push(pkg);
				continue;
			}
			virtual_pkgs.push(pkg);
		}
		assert!(!real_pkgs.is_empty());
		assert!(virtual_pkgs.is_empty());
	}

	#[test]
	fn virtual_pkgs() {
		let cache = Cache::new();

		// Test defaults and ensure there are no virtual packages.
		// And that we have any packages at all.
		let mut real_pkgs = Vec::new();
		let mut virtual_pkgs = Vec::new();

		let sort = PackageSort::default().virtual_pkgs();

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
	fn upgradable() {
		let cache = Cache::new();

		let sort = PackageSort::default().upgradable();
		for pkg in cache.packages(&sort) {
			assert!(pkg.is_upgradable())
		}
	}

	#[test]
	fn installed() {
		let cache = Cache::new();

		let sort = PackageSort::default().installed();
		for pkg in cache.packages(&sort) {
			assert!(pkg.is_installed())
		}
	}

	#[test]
	fn auto_installed() {
		let cache = Cache::new();

		let sort = PackageSort::default().auto_installed();
		for pkg in cache.packages(&sort) {
			assert!(pkg.is_auto_installed())
		}
	}

	#[test]
	fn auto_removable() {
		let cache = Cache::new();

		let sort = PackageSort::default().auto_removable();
		for pkg in cache.packages(&sort) {
			assert!(pkg.is_auto_removable())
		}
	}
}
