#[cfg(test)]
mod tests {
	use rust_apt::cache::*;

	#[test]
	fn mem_leak_test() {
		let cache = Cache::new();

		let sort = PackageSort::default().upgradable(true);

		for pkg in cache.packages() {
			println!("{}", pkg)
		}

		for pkg in cache.sorted(sort).values() {
			println!("This Package is Upgradable! {}", pkg.name);
			if let Some(candidate) = pkg.candidate() {
				println!("{candidate}");
			}
			if let Some(installed) = pkg.installed() {
				println!("{installed}");
			}
		}
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
		let sort = PackageSort::default().upgradable(true);

		assert!(sort.upgradable);
		assert!(!sort.virtual_pkgs);

		let sort = PackageSort::default().virtual_pkgs(true);

		assert!(!sort.upgradable);
		assert!(sort.virtual_pkgs);

		let sort = PackageSort::default().upgradable(true).virtual_pkgs(false);

		assert!(sort.upgradable);
		assert!(!sort.virtual_pkgs);
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
}
