mod depcache {
	use rust_apt::cache::{Cache, Upgrade};
	use rust_apt::package::Mark;

	#[test]
	fn mark_reinstall() {
		let cache = Cache::new();
		let pkg = cache.get("apt").unwrap();

		dbg!(pkg.marked_reinstall());
		dbg!(pkg.mark_reinstall(true));
		assert!(pkg.marked_reinstall());
	}

	#[test]
	fn mark_all() {
		// This test assumes that apt is installed
		let cache = Cache::new();
		let pkg = cache.get("apt").unwrap();

		let marks = [
			Mark::Keep,
			Mark::Auto,
			Mark::Manual,
			Mark::Remove,
			Mark::Purge,
			// Since apt is already installed these will not work
			// The only way they will is if it's able to be upgraded
			// Mark::Install,
			Mark::Reinstall,
			Mark::NoReinstall,
			// Mark::Upgrade,
		];

		// Set each mark, and then check the value based on the bool from setting.
		for mark in marks {
			if pkg.set(&mark) {
				assert!(pkg.state(&mark));
			} else {
				assert!(!pkg.state(&mark));
			}
			// Clear all the marks after each test
			// To ensure that the package states are clear
			cache.clear_marked().unwrap();
		}
	}

	#[test]
	fn upgrade() {
		// There isn't a great way to test if upgrade is working properly
		// as this is dynamic depending on the system.
		// This test will always pass, but print the status of the changes.
		// Occasionally manually compare the output to apt full-upgrade.
		let cache = Cache::new();
		cache.upgrade(&Upgrade::FullUpgrade).unwrap();

		for pkg in cache.get_changes(true) {
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
