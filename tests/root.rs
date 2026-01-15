mod root {
	use rust_apt::config::Config;
	use rust_apt::new_cache;
	use rust_apt::progress::{AcquireProgress, DynAcquireProgress, InstallProgress};
	use rust_apt::raw::{AcqTextStatus, ItemDesc, ItemState, PkgAcquire};
	use rust_apt::util::*;

	#[test]
	fn lock() {
		apt_lock().unwrap();
		apt_lock().unwrap();
		assert!(apt_is_locked());

		apt_unlock();
		assert!(apt_is_locked());

		apt_unlock();
		assert!(!apt_is_locked());
	}

	#[test]
	fn update() {
		struct Progress {}

		impl DynAcquireProgress for Progress {
			fn pulse_interval(&self) -> usize { 0 }

			fn hit(&mut self, item: &ItemDesc) {
				println!("\rHit:{} {}", item.owner().id(), item.description());
			}

			fn fetch(&mut self, item: &ItemDesc) {
				let mut string = format!("\rGet:{} {}", item.owner().id(), item.description());

				let file_size = item.owner().file_size();
				if file_size != 0 {
					string.push_str(&format!(" [{}]", unit_str(file_size, NumSys::Decimal)));
				}

				println!("{string}");
			}

			fn done(&mut self, _item: &ItemDesc) {}

			fn start(&mut self) {}

			fn stop(&mut self, owner: &AcqTextStatus) {
				if owner.fetched_bytes() != 0 {
					println!(
						"Fetched {} in {} ({}/s)",
						unit_str(owner.fetched_bytes(), NumSys::Decimal),
						time_str(owner.elapsed_time()),
						unit_str(owner.current_cps(), NumSys::Decimal)
					);
				} else {
					println!("Nothing to fetch.");
				}
			}

			fn fail(&mut self, item: &ItemDesc) {
				let mut show_error = true;
				let error_text = item.owner().error_text();
				let desc = format!("{} {}", item.owner().id(), item.description());

				match item.owner().status() {
					ItemState::StatIdle | ItemState::StatDone => {
						println!("\rIgn: {desc}");
						if error_text.is_empty()
							|| Config::new().bool("Acquire::Progress::Ignore::ShowErrorText", false)
						{
							show_error = false;
						}
					},
					_ => {
						println!("\rErr: {desc}");
					},
				}

				if show_error {
					println!("\r{error_text}");
				}
			}

			fn pulse(&mut self, _status: &AcqTextStatus, _owner: &PkgAcquire) {}
		}

		let cache = new_cache!().unwrap();

		// Test the default implementation for it
		let mut progress = AcquireProgress::apt();
		cache.update(&mut progress).unwrap();

		let cache = new_cache!().unwrap();

		// Test a new impl for AcquireProgress
		let mut progress = AcquireProgress::new(Progress {});
		cache.update(&mut progress).unwrap();
	}

	#[test]
	fn install_and_remove() {
		let debs = [
			"tests/files/cache/dep-pkg1_0.0.1.deb",
			"tests/files/cache/dep-pkg2_0.0.1.deb",
		];
		let cache = new_cache!(&debs).unwrap();
		let pkg = cache.get("dep-pkg2").unwrap();

		pkg.protect();
		pkg.mark_install(true, true);
		cache.resolve(false).unwrap();
		dbg!(pkg.marked_install());

		let mut progress = AcquireProgress::apt();
		let mut inst_progress = InstallProgress::apt();

		cache.commit(&mut progress, &mut inst_progress).unwrap();
		// After commit a new cache must be created for more operations
		let cache = new_cache!(&debs).unwrap();
		let pkg1 = cache.get("dep-pkg1").unwrap();
		let pkg2 = cache.get("dep-pkg2").unwrap();

		pkg1.mark_delete(true);
		pkg2.mark_delete(true);

		cache.commit(&mut progress, &mut inst_progress).unwrap();
	}

	#[test]
	fn install_with_debs() {
		let debs = [
			"tests/files/cache/dep-pkg1_0.0.1.deb",
			"tests/files/cache/dep-pkg2_0.0.1.deb",
		];
		let cache = new_cache!(&debs).unwrap();

		let pkg1 = cache.get("dep-pkg1").unwrap();
		let pkg2 = cache.get("dep-pkg2").unwrap();

		pkg1.mark_install(true, true);
		pkg2.mark_install(true, true);
		cache.resolve(false).unwrap();

		let mut progress = AcquireProgress::apt();
		let mut inst_progress = InstallProgress::apt();
		cache.commit(&mut progress, &mut inst_progress).unwrap();

		// You have to get a new cache after using commit.
		let cache = new_cache!(&debs).unwrap();

		// New packages will be required as well.
		let pkg1 = cache.get("dep-pkg1").unwrap();
		let pkg2 = cache.get("dep-pkg2").unwrap();

		// Leave no trace
		pkg1.mark_delete(true);
		pkg2.mark_delete(true);

		cache.commit(&mut progress, &mut inst_progress).unwrap();
	}
}
