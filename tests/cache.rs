mod cache {
	use std::collections::HashMap;
	use std::fmt::Write as _;

	use rust_apt::cache::*;
	use rust_apt::new_cache;
	use rust_apt::package::DepType;
	use rust_apt::util::*;

	#[test]
	fn test_raw_pkg() {
		let cache = new_cache!().unwrap();

		for pkg in cache.raw_pkgs().unwrap() {
			println!("ID: {}", pkg.id());
			println!("Name: {}", pkg.name());
			println!("Arch: {}", pkg.arch());
			println!("FullName: {}", pkg.fullname(false));
			println!("current_state: {}", pkg.current_state());
			println!("inst_state: {}", pkg.inst_state());
			println!("selected_state: {}\n", pkg.selected_state());

			match pkg.version_list() {
				Some(ver) => {
					println!("Version of '{}'", pkg.name());

					println!("    Version: {}", ver.version());
					println!("    Arch: {}", ver.arch());
					println!("    Section: {}", ver.section().unwrap_or_default());
					println!("    Source Pkg: {}", ver.source_name());
					println!("    Source Version: {}\n", ver.source_version());

					println!("End: {}\n\n", pkg.end());
				},
				None => {
					println!("'{}' is a Virtual Package\n", pkg.name());
				},
			}
		}
	}

	#[test]
	fn with_debs() {
		let cache = new_cache!(&[
			"tests/files/cache/apt.deb",
			"tests/files/cache/dep-pkg1_0.0.1.deb",
		])
		.unwrap();

		cache.get("apt").unwrap().get_version("5000:1.0.0").unwrap();
		cache.get("dep-pkg1").unwrap();

		assert!(new_cache!(&["tests/files/this-file-doesnt-exist.deb"]).is_err());

		// Check if it errors on a garbage empty file as well
		// signal: 11, SIGSEGV: invalid memory reference
		assert!(new_cache!(&["tests/files/cache/pkg.deb",]).is_err());
	}

	#[test]
	fn empty_deps() {
		// This would fail before https://gitlab.com/volian/rust-apt/-/merge_requests/29
		let cache = new_cache!().unwrap();
		let sort = PackageSort::default();

		// Iterate through all of the package and versions
		for pkg in cache.packages(&sort).unwrap() {
			for version in pkg.versions() {
				// Call depends_map to check for panic on null dependencies.
				version.depends_map();
			}
		}
	}

	#[test]
	fn parent_pkg() {
		let cache = new_cache!().unwrap();
		let pkg = cache.get("apt").unwrap();
		let version = pkg.versions().next().unwrap();
		let parent = version.parent();
		assert_eq!(pkg.id(), parent.id())
	}

	#[test]
	fn get_version() {
		let cache = new_cache!().unwrap();
		let pkg = cache.get("apt").unwrap();

		// The candidate for apt surely exists.
		let cand = pkg.candidate().unwrap();
		assert!(pkg.get_version(cand.version()).is_some());

		// I sure hope this doesn't exist.
		assert!(pkg.get_version("9.0.0.1").is_none());
	}

	#[test]
	fn all_packages() {
		let cache = new_cache!().unwrap();
		let sort = PackageSort::default();

		// All real packages should not be empty.
		assert!(cache.packages(&sort).unwrap().next().is_some());
		for pkg in cache.packages(&sort).unwrap() {
			// impl display??
			// println!("{pkg}")
			println!("{}:{}", pkg.name(), pkg.arch())
		}
	}

	#[test]
	fn descriptions() {
		let cache = new_cache!().unwrap();

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
		dbg!(inst_desc);
		dbg!(cand_desc);
	}

	#[test]
	fn version_uris() {
		let cache = new_cache!().unwrap();
		let pkg = cache.get("apt").unwrap();
		// Only test the candidate.
		// It's possible for the installed version to have no uris
		let cand = pkg.candidate().unwrap();
		assert!(cand.uris().next().is_some());
		dbg!(cand.uris().collect::<Vec<_>>());
	}

	#[test]
	fn depcache_marked() {
		let cache = new_cache!().unwrap();
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
		let cache = new_cache!().unwrap();
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
		let cache = new_cache!().unwrap();
		let sort = PackageSort::default();
		for pkg in cache.packages(&sort).unwrap() {
			assert!(!pkg.name().contains(':'))
		}
	}

	#[test]
	fn depends() {
		let cache = new_cache!().unwrap();

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
		assert!(cand.get_depends(&DepType::Replaces).is_some());
		// This test seems to work on Debian Sid desktop systems, but not in a Debian
		// Sid Docker container (and potentially other distros too). Leaving this
		// commented out until a solution is found.
		// assert!(cand.get_depends("Conflicts").is_some());
		assert!(cand.get_depends(&DepType::Breaks).is_some());

		// This part is basically just formatting an apt depends String
		// Like you would see in `apt show`
		let mut dep_str = String::new();
		dep_str.push_str("Depends: ");
		for dep in cand.depends_map().get(&DepType::Depends).unwrap() {
			if dep.is_or() {
				let mut or_str = String::new();
				let total = dep.base_deps.len() - 1;
				for (num, base_dep) in dep.base_deps.iter().enumerate() {
					or_str.push_str(base_dep.name());
					if let Some(comp) = base_dep.comp() {
						let _ = write!(or_str, "({} {})", comp, base_dep.version().unwrap());
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
				dep_str.push_str(lone_dep.name());

				if let Some(comp) = lone_dep.comp() {
					let _ = write!(dep_str, " ({} {})", comp, lone_dep.version().unwrap());
				}
				dep_str.push_str(", ");
			}
		}
		println!("{dep_str}");
	}

	#[test]
	fn test_hashmap() {
		let cache = new_cache!().unwrap();

		// clippy thinks that the package is mutable
		// But it only hashes the ID and you can't really mutate a version
		#[allow(clippy::mutable_key_type)]
		let mut pkg_map = HashMap::new();

		// clippy thinks that the version is mutable
		// But it only hashes the ID and you can't really mutate a version
		#[allow(clippy::mutable_key_type)]
		let mut ver_map = HashMap::new();

		let sort = PackageSort::default();

		// Iterate the package cache and add them to a hashmap
		for pkg in cache.packages(&sort).unwrap() {
			let value = pkg.arch().to_string();
			pkg_map.insert(pkg, value);
		}

		// Iterate the package map and add all the candidates into a hashmap
		for (pkg, _arch) in pkg_map.iter() {
			if let Some(cand) = pkg.candidate() {
				let value = cand.arch().to_string();
				ver_map.insert(cand, value);
			}
		}
		// Doesn't need an assert. It won't compile
		// if the structs can't go into a hashmap
	}

	#[test]
	fn parent_dep() {
		let cache = new_cache!().unwrap();
		let sort = PackageSort::default();

		for pkg in cache.packages(&sort).unwrap() {
			// Iterate over the reverse depends
			// Iterating rdepends could segfault.
			// See: https://gitlab.com/volian/rust-apt/-/merge_requests/36
			for deps in pkg.rdepends_map().values() {
				for dep in deps {
					let base_dep = dep.first();
					// Reverse Dependencies always have a version
					base_dep.version().unwrap();
				}
			}

			// There should be a candidate to iterate its regular deps
			if let Some(cand) = pkg.candidate() {
				if let Some(deps) = cand.dependencies() {
					for dep in &deps {
						let base_dep = dep.first();
						// Regular deps do not always have a version
						base_dep.version();
					}
				}
			}
		}
	}

	#[test]
	fn provides_list() {
		let cache = new_cache!().unwrap();
		let pkg = cache.get("apt").unwrap();
		let cand = pkg.candidate().unwrap();
		let provides_list = cand.provides_list().unwrap().collect::<Vec<_>>();

		assert!(provides_list.len() == 1);
		// 'apt' seems to always provide for 'apt-transport-https' at APT's version.
		// If it ever doesn't, this test will break.
		let provide = provides_list.get(0).unwrap();
		assert!(provide.name() == "apt-transport-https");
		assert!(provide.version_str().unwrap() == cand.version());
	}

	// This Test is for https://gitlab.com/volian/rust-apt/-/issues/24
	// TODO: refactor and enable this test so it can run in the CI to make sure we
	// don't regress. We need to get the lists dir from the apt config, and then
	// maybe pick a random InRelease file Back that up, do the editing and then
	// restore it at the end of the test. cache.packages should be an error and not
	// segfault.
	//
	// #[test]
	// fn test_segfault() {
	// 	use std::io::Write;

	// 	let mut f = std::fs::OpenOptions::new()
	// 		.write(true)
	// 		.append(true)
	// 		.open("/var/lib/apt/lists/deb.debian.org_debian_dists_sid_InRelease")
	// 		.unwrap();

	// 	f.write_all(b"\ndsadasdasdas\n").unwrap();
	// 	f.flush().unwrap();

	// 	drop(f);

	// 	let cache = new_cache!().unwrap();

	// 	let sort = PackageSort::default();

	// assert!(cache.packages(&sort).is_err())

	/// This test is tied pretty closely to the currently available versions in
	/// the Ubuntu/Debian repos. Feel free to adjust if you can verify its
	/// needed.
	#[test]
	fn rev_provides_list() {
		// Test concrete packages with provides.
		let cache = new_cache!().unwrap();
		let apt = cache.get("apt").unwrap();
		let ver = apt.candidate().unwrap();
		let pkg = cache.get("apt-transport-https").unwrap();

		{
			let mut rev_provides_list = pkg.provides_list().unwrap();
			let provides_pkg = rev_provides_list.next().unwrap();

			assert!(rev_provides_list.next().is_none());

			let parent = provides_pkg.target_pkg();
			assert!(parent.name().contains("apt"));
			assert_eq!(provides_pkg.version_str().unwrap(), ver.version());
		}

		{
			let mut rev_provides_list =
				pkg.provides_list()
					.unwrap()
					.filter(|p| match p.version_str() {
						Err(_) => false,
						Ok(version) => version == ver.version(),
					});

			let provides_pkg = rev_provides_list.next().unwrap();
			assert!(rev_provides_list.next().is_none());

			let parent = provides_pkg.target_pkg();
			assert_eq!(parent.name(), "apt");

			assert_eq!(provides_pkg.version_str().unwrap(), ver.version());
		}

		{
			let mut rev_provides_list =
				pkg.provides_list()
					.unwrap()
					.filter(|p| match p.version_str() {
						Err(_) => false,
						Ok(version) => version == "50000000000.0.0",
					});
			assert!(rev_provides_list.next().is_none());
		}

		// Test a virtual package with provides.
		{
			let pkg = cache.get("www-browser").unwrap();
			assert!(pkg.provides_list().unwrap().next().is_some());
		}
	}

	#[test]
	fn sources() {
		let cache = new_cache!().unwrap();
		// If the source lists don't exists there is problems.
		assert!(!cache.source_uris().is_empty());
	}

	#[test]
	fn cache_count() {
		let cache = new_cache!().unwrap();
		match cache.depcache().disk_size() {
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
		let cache = new_cache!().unwrap();
		let apt = cache.get("apt").unwrap();
		let dpkg = cache.get("dpkg").unwrap();

		let apt_ver = apt.candidate().unwrap();
		let dpkg_ver = dpkg.candidate().unwrap();

		assert!(apt_ver > dpkg_ver);
		assert!(dpkg_ver < apt_ver);
		assert!(apt_ver != dpkg_ver);
	}

	#[test]
	// This test relies on 'gobby' and 'gsasl-common' not being installed.
	fn good_resolution() {
		let cache = new_cache!().unwrap();
		let pkg = cache.get("gobby").unwrap();

		pkg.mark_install(true, true);
		pkg.protect();
		cache.resolve(false).unwrap();

		let pkg2 = cache.get("gsasl-common").unwrap();
		pkg2.mark_install(true, true);
		assert!(pkg2.marked_install())
	}

	// For now `zeek` has broken dependencies so the resolver errors.
	// If this test fails, potentially find a reason.
	#[test]
	fn bad_resolution() {
		let cache = new_cache!().unwrap();

		let pkg = cache.get("zeek").unwrap();

		pkg.mark_install(false, true);
		pkg.protect();

		assert!(cache.resolve(false).is_err());
	}

	#[test]
	fn depcache_clear() {
		let cache = new_cache!().unwrap();
		let pkg = cache.get("apt").unwrap();

		pkg.mark_delete(true);

		assert!(pkg.marked_delete());

		cache.depcache().clear_marked().unwrap();
		assert!(!pkg.marked_delete());
	}

	#[test]
	fn origins() {
		let cache = new_cache!().unwrap();
		let apt = cache.get("apt").unwrap();
		let apt_ver = apt.candidate().unwrap();
		let pkg_files = apt_ver.package_files().collect::<Vec<_>>();

		// Package files should not be empty if we got a candidate from `apt`.
		assert!(!pkg_files.is_empty());

		for mut pkg_file in pkg_files {
			// Apt should have all of these blocks in the package file.
			assert!(pkg_file.filename().is_ok());
			assert!(pkg_file.archive().is_ok());

			// If the archive is `/var/lib/dpkg/status` These will be None.
			if pkg_file.archive().unwrap() != "now" {
				assert!(pkg_file.origin().is_ok());
				assert!(pkg_file.codename().is_ok());
				assert!(pkg_file.label().is_ok());
				assert!(pkg_file.site().is_ok());
				assert!(pkg_file.arch().is_ok());
			}

			// These should be okay regardless.
			assert!(pkg_file.component().is_ok());
			assert!(pkg_file.index_type().is_ok());

			// Index should not be 0.
			assert_ne!(pkg_file.index(), 0);

			// Apt should likely be from a trusted repository.
			assert!(cache.is_trusted(&mut pkg_file));
			// Print it in case I want to see.
			// println!("{pkg_file}");
		}
	}
}
