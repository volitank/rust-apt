#[cfg(test)]
mod cache {
	use std::fmt::Write as _;

	use rust_apt::cache::*;
	use rust_apt::util::*;

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
	fn parent_pkg() {
		let cache = Cache::new();
		let pkg = cache.get("apt").unwrap();
		let version = pkg.versions().next().unwrap();
		let parent = version.parent();
		assert_eq!(pkg.id(), parent.id())
	}

	#[test]
	fn all_packages() {
		let cache = Cache::new();
		let sort = PackageSort::default();

		// All real packages should not be empty.
		assert!(cache.packages(&sort).next().is_some());
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
		assert!(cand.uris().next().is_some());
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
		// Apt could be installed, and the package no longer exists
		// In the cache. For this case we grab the candidate so it won't fail.
		let version = pkg.candidate().unwrap();
		assert!(version.sha256().is_some());
		assert!(version.hash("sha256").is_some());
		assert!(version.sha512().is_none());
		assert!(version.hash("md5sum").is_some());
		assert!(version.hash("sha1").is_none())
	}

	#[test]
	fn shortname() {
		let cache = Cache::new();
		let sort = PackageSort::default();
		for pkg in cache.packages(&sort) {
			assert!(!pkg.name().contains(':'))
		}
	}

	#[test]
	fn provides() {
		let cache = Cache::new();
		if let Some(pkg) = cache.get("www-browser") {
			assert!(cache.provides(&pkg, true).next().is_some());
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
				assert!(dep.all_targets().next().is_some());
			}
		}
		assert!(cand.recommends().is_some());
		assert!(cand.suggests().is_some());
		// TODO: Add these as methods
		assert!(cand.get_depends("Replaces").is_some());
		// This test seems to work on Debian Sid desktop systems, but not in a Debian
		// Sid Docker container (and potentially other distros too). Leaving this
		// commented out until a solution is found.
		// assert!(cand.get_depends("Conflicts").is_some());
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
						let _ = write!(or_str, "({} {})", base_dep.comp(), base_dep.version(),);
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
					let _ = write!(dep_str, " ({} {})", lone_dep.comp(), lone_dep.version(),);
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
		assert!(cache.sources().next().is_some());
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
	fn test_unit_str() {
		let testcase = [
			(1649267441664_u64, "1.50 TiB", "1.65 TB"),
			(1610612736_u64, "1.50 GiB", "1.61 GB"),
			(1572864_u64, "1.50 MiB", "1.57 MB"),
			(1536_u64, "1.50 KiB", "1.54 KB"),
			(1024_u64, "1024 B", "1.02 KB"),
			(1_u64, "1 B", "1 B"),
		];

		for (num, binary, decimal) in testcase {
			assert_eq!(binary, unit_str(num, NumSys::Binary));
			assert_eq!(decimal, unit_str(num, NumSys::Decimal));
		}
	}

	#[test]
	// This test relies on the version of 'apt' being higher than 'dpkg'.
	fn version_comparisons() {
		let cache = Cache::new();
		let apt_ver = cache.get("apt").unwrap().candidate().unwrap();
		let dpkg_ver = cache.get("dpkg").unwrap().candidate().unwrap();
		assert!(apt_ver > dpkg_ver);
		assert!(dpkg_ver < apt_ver);
		assert!(apt_ver != dpkg_ver);
	}
}

#[cfg(test)]
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

mod config {
	use rust_apt::config::Config;

	#[test]
	fn clear() {
		// Test to make sure that the config populates properly.
		// Config will be empty if it hasn't been initialized.
		let config = Config::new_clear();
		config.clear_all();

		let empty_config = config.find("APT::Architecture", "");
		assert!(!config.contains("APT::Architecture"));
		assert!(empty_config.is_empty());

		// Reset the configuration which will clear and reinit.
		config.reset();

		// Now it should NOT be empty.
		let config_dump = config.find("APT::Architecture", "");
		assert!(config.contains("APT::Architecture"));
		assert!(!config_dump.is_empty());
		println!("{}", config.dump());
	}

	#[test]
	fn find_and_set() {
		let config = Config::new_clear();
		let key = "rust_apt::NotExist";

		// Find our key. It should not exist.
		assert_eq!(config.find(key, "None"), "None");

		// Set the key to something.
		config.set(key, "Exists!");

		// Find again and it should be there.
		assert_eq!(config.find(key, "None"), "Exists!");

		// Test other find functions on known defaults.
		assert!(!config.bool("APT::Install-Suggests", true));
		assert_eq!(config.int("APT::Install-Suggests", 20), 0);
		assert_eq!(
			config.file("Dir::Cache::pkgcache", ""),
			"/var/cache/apt/pkgcache.bin"
		);
		assert_eq!(
			config.dir("Dir::Etc::sourceparts", ""),
			"/etc/apt/sources.list.d/"
		);

		// Check if we can set a configuration list and retrieve it.
		// Make sure that the target vector is empty.
		assert!(config.find_vector("rust_apt::aptlist").is_empty());

		// Now fill our configuration vector and set it.
		let apt_list = vec!["this", "is", "my", "apt", "list"];
		config.set_vector("rust_apt::aptlist", &apt_list);

		// Retrieve a new vector from the configuration.
		let apt_vector = config.find_vector("rust_apt::aptlist");

		// If everything went smooth, our original vector should match the new one
		assert_eq!(apt_list, apt_vector);

		// Now test if we can remove a single value from the list.
		config.clear_value("rust_apt::aptlist", "my");

		// This will let us know if it worked!
		assert_eq!(
			config.find_vector("rust_apt::aptlist"),
			vec!["this", "is", "apt", "list"]
		);

		// Finally test and see if we can clear the entire list.
		config.clear("rust_apt::aptlist");
		assert!(config.find_vector("rust_apt::aptlist").is_empty());
	}
}

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

/// Tests that require root
mod root {
	use rust_apt::cache::*;
	use rust_apt::progress::{raw, AptUpdateProgress, UpdateProgress};
	use rust_apt::util::*;

	#[test]
	fn update() {
		let cache = Cache::new();
		struct Progress {}

		impl UpdateProgress for Progress {
			fn pulse_interval(&self) -> usize { 0 }

			fn hit(&mut self, id: u32, description: String) {
				println!("\rHit:{} {}", id, description);
			}

			fn fetch(&mut self, id: u32, description: String, file_size: u64) {
				if file_size != 0 {
					println!(
						"\rGet:{id} {description} [{}]",
						unit_str(file_size, NumSys::Decimal)
					);
				} else {
					println!("\rGet:{id} {description}");
				}
			}

			fn done(&mut self) {}

			fn start(&mut self) {}

			fn stop(
				&mut self,
				fetched_bytes: u64,
				elapsed_time: u64,
				current_cps: u64,
				_pending_errors: bool,
			) {
				if fetched_bytes != 0 {
					println!(
						"Fetched {} in {} ({}/s)",
						unit_str(fetched_bytes, NumSys::Decimal),
						time_str(elapsed_time),
						unit_str(current_cps, NumSys::Decimal)
					);
				} else {
					println!("Nothing to fetch.");
				}
			}

			fn fail(&mut self, id: u32, description: String, status: u32, error_text: String) {
				let mut show_error = true;

				if status == 0 || status == 2 {
					println!("\rIgn: {id} {description}");
					if error_text.is_empty() {
						show_error = false;
					}
				} else {
					println!("\rErr: {id} {description}");
				}
				if show_error {
					println!("\r{error_text}");
				}
			}

			fn pulse(
				&mut self,
				_workers: Vec<raw::Worker>,
				_percent: f32,
				_total_bytes: u64,
				_current_bytes: u64,
				_current_cps: u64,
			) {
			}
		}

		// Test a new impl for UpdateProgress
		let mut progress: Box<dyn UpdateProgress> = Box::new(Progress {});
		cache.update(&mut progress).unwrap();

		// Test the default implementation for it
		let mut progress: Box<dyn UpdateProgress> = Box::new(AptUpdateProgress::new());
		cache.update(&mut progress).unwrap();
	}
}
