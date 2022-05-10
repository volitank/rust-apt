#[cfg(test)]
mod tests {
	use rust_apt::cache::*;

	#[test]
	fn mem_leak_test() {
		let cache = Cache::new();

		let sort = PackageSort {
			upgradable: true,
			virtual_pkgs: false,
		};

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
			println!("{:?}\n", version.get_uris());
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
	fn test_fields() {
		let cache = Cache::new();
		if let Some(apt) = cache.get("apt") {
			println!("{apt}");
			for version in apt.versions() {
				println!("{version}")
			}
		};
	}
}
