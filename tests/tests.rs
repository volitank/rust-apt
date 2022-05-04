#[cfg(test)]
mod tests {
	use rust_apt::cache::*;

	#[test]
	fn mem_leak_test() {
		let cache = Cache::new();
		println!("Cache Initialized");

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
				println!("{}", candidate);
			}
			if let Some(installed) = pkg.installed() {
				println!("{}", installed);
			}
		}
		let mut versions = Vec::new();
		// if let Some(nala) = cache.get("nala") {
		// 	//drop(cache);
		// 	println!("{}", nala.name);
		// 	for version in nala.versions() {
		// 		println!("{}", version);
		// 		versions.push(version);
		// 	}
		// }
		//drop(cache);
		if let Some(apt) = cache.get("apt") {
			println!("{}", apt.name);
			for version in apt.versions() {
				println!("{}", version);
				versions.push(version);
			}
		}

		//drop(cache);
		for version in versions {
			println!("{}", version);
			println!("{:?}\n", version.get_uris());
		}
		// if let Some(nala) = cache.get("nala") {
		// 	println!("{}", nala.name);
		// 	for version in nala.versions() {
		// 		println!("{}", version)
		// 	}
		// }
		println!("Done!");
	}
}
