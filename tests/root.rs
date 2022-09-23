mod root {
	use rust_apt::cache::*;
	use rust_apt::progress::{raw, AcquireProgress, AptAcquireProgress, AptInstallProgress};
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
		let cache = Cache::new();
		struct Progress {}

		impl AcquireProgress for Progress {
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

		// Test a new impl for AcquireProgress
		let mut progress: Box<dyn AcquireProgress> = Box::new(Progress {});
		cache.update(&mut progress).unwrap();

		// Test the default implementation for it
		let mut progress = AptAcquireProgress::new_box();
		cache.update(&mut progress).unwrap();
	}

	#[test]
	fn install_and_remove() {
		let cache = Cache::new();

		let pkg = cache.get("neofetch").unwrap();

		pkg.protect();
		pkg.mark_install(true, true);
		cache.resolve(false).unwrap();
		dbg!(pkg.marked_install());

		let mut progress = AptAcquireProgress::new_box();
		let mut inst_progress = AptInstallProgress::new_box();

		cache.commit(&mut progress, &mut inst_progress).unwrap();
		// After commit a new cache must be created for more operations
		cache.clear().unwrap();

		// Segmentation fault if the cache isn't remapped properly
		pkg.mark_delete(true);

		cache.commit(&mut progress, &mut inst_progress).unwrap();
	}

	#[test]
	fn install_with_debs() {
		let cache = Cache::debs(&[
			"tests/files/cache/dep-pkg1_0.0.1.deb",
			"tests/files/cache/dep-pkg2_0.0.1.deb",
		])
		.unwrap();

		let pkg1 = cache.get("dep-pkg1").unwrap();
		let pkg2 = cache.get("dep-pkg2").unwrap();

		pkg1.mark_install(true, true);
		pkg2.mark_install(true, true);
		cache.resolve(false).unwrap();

		let mut progress = AptAcquireProgress::new_box();
		let mut inst_progress = AptInstallProgress::new_box();
		cache.commit(&mut progress, &mut inst_progress).unwrap();

		cache.clear().unwrap();

		// Leave no trace
		pkg1.mark_delete(true);
		pkg2.mark_delete(true);

		cache.commit(&mut progress, &mut inst_progress).unwrap();
	}
}
