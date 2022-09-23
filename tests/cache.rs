mod cache {
	use std::fmt::Write as _;

	use rust_apt::cache::*;
	use rust_apt::util::*;

	#[test]
	fn with_debs() {
		let cache = Cache::debs(&[
			"tests/files/cache/apt.deb",
			"tests/files/cache/dep-pkg1_0.0.1.deb",
		])
		.unwrap();
		cache.get("apt").unwrap().get_version("5000:1.0.0").unwrap();
		cache.get("dep-pkg1").unwrap();

		assert!(Cache::debs(&["tests/files/this-file-doesnt-exist.deb"]).is_err());
	}

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
	fn get_version() {
		let cache = Cache::new();
		let pkg = cache.get("apt").unwrap();

		// The candidate for apt surely exists.
		let cand_str = pkg.candidate().unwrap().version();
		assert!(pkg.get_version(&cand_str).is_some());

		// I sure hope this doesn't exist.
		assert!(pkg.get_version("9.0.0.1").is_none());
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
	fn provides_list() {
		let cache = Cache::new();
		let pkg = cache.get("apt").unwrap().candidate().unwrap();
		let provides_list = pkg.provides_list();
		let (provides_pkgname, provides_pkgver) = provides_list.get(0).unwrap();

		assert!(provides_list.len() == 1);
		// 'apt' seems to always provide for 'apt-transport-https' at APT's version. If
		// it ever doesn't, this test will break.
		assert!(provides_pkgname == "apt-transport-https");
		assert!(provides_pkgver.as_ref().unwrap() == &pkg.version());
	}

	/// This test is tied pretty closely to the currently available versions in
	/// the Ubuntu/Debian repos. Feel free to adjust if you can verify its
	/// needed.
	#[test]
	fn rev_provides_list() {
		// Test concrete packages with provides.
		let cache = Cache::new();
		let ver = cache.get("apt").unwrap().candidate().unwrap();
		let pkg = cache.get("apt-transport-https").unwrap();

		{
			let rev_provides_list = pkg.rev_provides_list(None);
			let provides_pkg = rev_provides_list.get(0).unwrap();
			let mut prov_names = Vec::new();
			for pkg in &rev_provides_list {
				prov_names.push(pkg.parent().name());
			}

			// This function is wild and possibly can fail all the time.
			// For example adding i386 arch will cause this to fail.
			assert_eq!(rev_provides_list.len(), 1);
			assert!(prov_names.contains(&"apt".to_string()));
			assert_eq!(provides_pkg.version(), ver.version());
		}

		{
			let rev_provides_list = pkg.rev_provides_list(Some(("=", &ver.version())));
			let provides_pkg = rev_provides_list.get(0).unwrap();
			let mut prov_names = Vec::new();
			for pkg in &rev_provides_list {
				prov_names.push(pkg.parent().name());
			}
			assert_eq!(rev_provides_list.len(), 1);
			assert!(prov_names.contains(&"apt".to_string()));
			assert_eq!(provides_pkg.version(), ver.version());
		}

		{
			let rev_provides_list = pkg.rev_provides_list(Some(("=", "50000000000.0.0")));
			assert_eq!(rev_provides_list.len(), 0);
		}

		// Test a virtual package with provides.
		{
			let pkg = cache.get("www-browser").unwrap();
			assert!(!pkg.rev_provides_list(None).is_empty());
		}
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

	#[test]
	// This test relies on 'gobby' and 'gsasl-common' not being installed.
	fn good_resolution() {
		let cache = Cache::new();
		let pkg = cache.get("gobby").unwrap();

		pkg.mark_install(true, true);
		pkg.protect();
		cache.resolve(false).unwrap();

		let pkg2 = cache.get("gsasl-common").unwrap();
		pkg2.mark_install(true, true);
		assert!(pkg2.marked_install())
	}

	// For now `zorp` has broken dependencies so the resolver errors.
	// If this test fails, potentially find a reason.
	#[test]
	fn bad_resolution() {
		let cache = Cache::new();

		let pkg = cache.get("zorp").unwrap();

		pkg.mark_install(false, true);
		pkg.protect();

		assert!(cache.resolve(false).is_err());
	}

	#[test]
	fn depcache_clear() {
		let cache = Cache::new();
		let pkg = cache.get("apt").unwrap();

		pkg.mark_delete(true);

		assert!(pkg.marked_delete());

		cache.clear_marked().unwrap();
		assert!(!pkg.marked_delete());
	}

	#[test]
	fn cache_remap() {
		let cache = Cache::new();
		let pkg = cache.get("apt").unwrap();
		let cand = pkg.candidate().unwrap();

		cache.clear().unwrap();

		// These will segfault if the remap isn't done properly
		dbg!(pkg.mark_delete(true));
		dbg!(cand.version());
		dbg!(cand);
	}

	#[test]
	fn origins() {
		let cache = Cache::new();
		let apt_ver = cache.get("apt").unwrap().candidate().unwrap();
		let pkg_files = apt_ver.package_files().collect::<Vec<_>>();

		// Package files should not be empty if we got a candidate from `apt`.
		assert!(!pkg_files.is_empty());

		for pkg_file in &pkg_files {
			// Apt should have all of these blocks in the package file.
			assert!(pkg_file.filename().is_some());
			assert!(pkg_file.archive().is_some());

			// If the archive is `/var/lib/dpkg/status` These will be None.
			if pkg_file.archive().unwrap() != "now" {
				assert!(pkg_file.origin().is_some());
				assert!(pkg_file.codename().is_some());
				assert!(pkg_file.label().is_some());
				assert!(pkg_file.site().is_some());
				assert!(pkg_file.arch().is_some());
			}

			// These should be okay regardless.
			assert!(pkg_file.component().is_some());
			assert!(pkg_file.index_type().is_some());

			// Index should not be 0.
			assert_ne!(pkg_file.index(), 0);

			// Apt should likely be from a trusted repository.
			assert!(pkg_file.is_trusted());
			// Print it in case I want to see.
			println!("{pkg_file}");
		}
	}
}
