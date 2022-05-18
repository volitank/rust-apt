#[cfg(test)]
mod tests {
	use rust_apt::cache::*;

	#[test]
	fn test_version_vec() {
		let cache = Cache::new();

		let mut versions = Vec::new();
		if let Some(apt) = cache.get("apt") {
			println!("{}", apt.name);
			for version in apt.versions() {
				println!("{version}");
				versions.push(version);
			}
		}

		for version in versions {
			println!("{version}");
			println!("Version is installed? {}", version.is_installed());
			println!("{:?}\n", version.uris().collect::<Vec<_>>());
		}
	}

	#[test]
	fn test_all_packages() {
		let cache = Cache::new();
		let sort = PackageSort::default();

		for pkg in cache.packages(&sort) {
			println!("{pkg}")
		}
	}

	#[test]
	fn test_upgradable() {
		let cache = Cache::new();
		let sort = PackageSort::default().upgradable(true);

		for pkg in cache.sorted(&sort) {
			println!(
				"Package is Upgradable! {} ({}) -> ({})",
				pkg.name,
				pkg.installed().unwrap().version,
				pkg.candidate().unwrap().version
			);
		}
	}

	#[test]
	fn test_installed() {
		let cache = Cache::new();
		let sort = PackageSort::default().installed(true);

		for pkg in cache.sorted(&sort) {
			println!(
				"Package is Installed! {} ({})",
				pkg.name,
				pkg.installed().unwrap().version
			);
		}
	}

	#[test]
	fn test_descriptions() {
		let cache = Cache::new();
		if let Some(apt) = cache.get("apt") {
			if let Some((cand, inst)) = apt.candidate().zip(apt.installed()) {
				println!("Package Name: {}", apt.name);
				println!(
					"Summary: {}\nDescription:\n\n{}\n",
					cand.summary(),
					cand.description()
				);
				println!(
					"Summary: {}\nDescription:\n\n{}\n",
					inst.summary(),
					inst.description()
				);
			}
		};
	}

	#[test]
	fn test_version() {
		let cache = Cache::new();
		println!("Package and Version Test:");
		if let Some(apt) = cache.get("apt") {
			println!("{apt}");
			for version in apt.versions() {
				println!("{version}");
				for uri in version.uris() {
					println!("{uri}")
				}
			}
		};
	}

	#[test]
	fn sort_defaults() {
		let sort = PackageSort::default().upgradable(true).installed(true);

		assert!(sort.upgradable);
		assert!(!sort.virtual_pkgs);
		assert!(sort.installed);

		let sort = PackageSort::default().virtual_pkgs(true);

		assert!(!sort.upgradable);
		assert!(sort.virtual_pkgs);
		assert!(!sort.installed);

		let sort = PackageSort::default()
			.upgradable(true)
			.virtual_pkgs(false)
			.installed(true);

		assert!(sort.upgradable);
		assert!(!sort.virtual_pkgs);
		assert!(sort.installed);
	}

	#[test]
	fn test_hashes() {
		let cache = Cache::new();
		if let Some(apt) = cache.get("apt") {
			for version in apt.versions() {
				println!("sha256 {:?}", version.sha256());
				println!("sha256 {:?}", version.hash("sha256"));
				println!("sha512 {:?}", version.sha512());
				println!("md5sum {:?}", version.hash("md5sum"));
				println!("sha1 {:?}", version.hash("sha1"))
			}
		};
	}

	#[test]
	fn test_sources() {
		let cache = Cache::new();
		for source in cache.sources() {
			println!("uri: {}, filename: {}", source.uri, source.filename);
		}
	}
}
