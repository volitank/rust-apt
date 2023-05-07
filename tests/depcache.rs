mod depcache {
	use rust_apt::cache::Upgrade;
	use rust_apt::new_cache;
	// use rust_apt::package::Mark;

	#[test]
	fn mark_reinstall() {
		let cache = new_cache!().unwrap();
		let pkg = cache.get("apt").unwrap();

		dbg!(pkg.marked_reinstall());
		dbg!(pkg.mark_reinstall(true));
		assert!(pkg.marked_reinstall());
	}

	#[test]
	fn action_groups() {
		let cache = new_cache!().unwrap();
		let action_group = cache.depcache().action_group();

		// The C++ deconstructor will be run when the action group leaves scope.
		action_group.release();
	}

	// Make a test for getting the candidate after you set a candidate.
	// Make sure it's the expected version.
	// We had to change to getting the candidate from the depcache.
	// https://gitlab.com/volian/rust-apt/-/issues/14

	// #[test]
	// fn changes_test() {
	// 	let cache = new_cache!().unwrap();

	// 	let pkg = cache.get("nala").unwrap();

	// 	let ver = pkg.get_version("0.12.1").unwrap();

	// 	ver.set_candidate();

	// 	let cand = pkg.candidate().unwrap();

	// 	println!("Version is {}", cand.version())
	// }

	#[test]
	fn upgrade() {
		// There isn't a great way to test if upgrade is working properly
		// as this is dynamic depending on the system.
		// This test will always pass, but print the status of the changes.
		// Occasionally manually compare the output to apt full-upgrade.
		let cache = new_cache!().unwrap();
		cache.upgrade(&Upgrade::FullUpgrade).unwrap();

		for pkg in cache.get_changes(true).unwrap() {
			if pkg.marked_install() {
				println!("{} is marked install", pkg.name());
				// If the package is marked install then it will also
				// show up as marked upgrade, downgrade etc.
				// Check this first and continue.
				continue;
			}
			if pkg.marked_upgrade() {
				println!("{} is marked upgrade", pkg.name())
			}
			if pkg.marked_delete() {
				println!("{} is marked remove", pkg.name())
			}
			if pkg.marked_reinstall() {
				println!("{} is marked reinstall", pkg.name())
			}
			if pkg.marked_downgrade() {
				println!("{} is marked downgrade", pkg.name())
			}
		}
	}
}
