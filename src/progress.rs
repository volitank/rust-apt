//! Contains Progress struct for updating the package list.
use std::fmt::Write as _;
use std::io::{stdout, Write};

use termsize;

use crate::raw::apt;
use crate::util::{time_str, unit_str, NumSys};

/// Trait you can impl on any struct to customize the output of the update.
pub trait UpdateProgress {
	/// Called on c++ to set the pulse interval.
	fn pulse_interval(&self) -> usize;

	/// Called when an item is confirmed to be up-to-date.
	fn hit(&mut self, id: u32, description: String);

	/// Called when an Item has started to download
	fn fetch(&mut self, id: u32, description: String, file_size: u64);

	/// Called when an Item fails to download
	fn fail(&mut self, id: u32, description: String, status: u32, error_text: String);

	/// Called periodically to provide the overall progress information
	fn pulse(
		&mut self,
		workers: Vec<apt::Worker>,
		percent: f32,
		total_bytes: u64,
		current_bytes: u64,
		current_cps: u64,
	);

	/// Called when an item is successfully and completely fetched.
	fn done(&mut self);

	/// Called when progress has started
	fn start(&mut self);

	/// Called when progress has finished
	fn stop(
		&mut self,
		fetched_bytes: u64,
		elapsed_time: u64,
		current_cps: u64,
		pending_errors: bool,
	);
}

// TODO: Make better structs for pkgAcquire items, workers, owners.
/// AptUpdateProgress is the default struct for the update method on the cache.
///
/// This struct mimics the output of `apt update`.
#[derive(Default, Debug)]
pub struct AptUpdateProgress {
	lastline: usize,
	pulse_interval: usize,
	disable: bool,
}

impl AptUpdateProgress {
	/// Returns a new default progress instance.
	pub fn new() -> Self { Self::default() }

	/// Returns a disabled progress instance. No output will be shown.
	pub fn disable() -> Self {
		AptUpdateProgress {
			disable: true,
			..Default::default()
		}
	}

	/// Returns the current terminal width or the default of 80
	/// One is taken away to account for the cursor
	fn screen_width(&self) -> usize {
		if let Some(size) = termsize::get() {
			return usize::from(size.cols - 1);
		}
		80 - 1
	}

	/// Helper function to clear the last line.
	fn clear_last_line(&mut self, term_width: usize) {
		if self.disable {
			return;
		}

		if self.lastline == 0 {
			return;
		}

		if self.lastline > term_width {
			self.lastline = term_width
		}

		print!("\r{}", " ".repeat(self.lastline));
		print!("\r");
		stdout().flush().unwrap();
	}
}

impl UpdateProgress for AptUpdateProgress {
	/// Used to send the pulse interval to the apt progress class.
	///
	/// Pulse Interval is in microseconds.
	///
	/// Example: 1 second = 1000000 microseconds.
	///
	/// Apt default is 500000 microseconds or 0.5 seconds.
	///
	/// The higher the number, the less frequent pulse updates will be.
	///
	/// Pulse Interval set to 0 assumes the apt defaults.
	fn pulse_interval(&self) -> usize { self.pulse_interval }

	/// Called when an item is confirmed to be up-to-date.
	///
	/// Prints out the short description and the expected size.
	fn hit(&mut self, id: u32, description: String) {
		if self.disable {
			return;
		}

		self.clear_last_line(self.screen_width());

		println!("\rHit:{} {}", id, description);
	}

	/// Called when an Item has started to download
	///
	/// Prints out the short description and the expected size.
	fn fetch(&mut self, id: u32, description: String, file_size: u64) {
		if self.disable {
			return;
		}

		self.clear_last_line(self.screen_width());

		if file_size != 0 {
			println!(
				"\rGet:{id} {description} [{}]",
				unit_str(file_size, NumSys::Decimal)
			);
		} else {
			println!("\rGet:{id} {description}");
		}
	}

	/// Called when an item is successfully and completely fetched.
	///
	/// We don't print anything here to remain consistent with apt.
	///
	/// TODO: Pass through information here.
	/// Likely when we make a general struct fork the items.
	fn done(&mut self) {
		// self.clear_last_line(self.screen_width());

		// println!("This is done!");
	}

	/// Called when progress has started.
	///
	/// Start does not pass information into the method.
	///
	/// We do not print anything here to remain consistent with apt.
	/// lastline length is set to 0 to ensure consistency when progress begins.
	fn start(&mut self) { self.lastline = 0; }

	/// Called when progress has finished.
	///
	/// Stop does not pass information into the method.
	///
	/// prints out the bytes downloaded and the overall average line speed.
	fn stop(
		&mut self,
		fetched_bytes: u64,
		elapsed_time: u64,
		current_cps: u64,
		pending_errors: bool,
	) {
		if self.disable {
			return;
		}

		self.clear_last_line(self.screen_width());

		if pending_errors {
			return;
		}

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

	/// Called when an Item fails to download.
	///
	/// Print out the ErrorText for the Item.
	fn fail(&mut self, id: u32, description: String, status: u32, error_text: String) {
		if self.disable {
			return;
		}

		self.clear_last_line(self.screen_width());

		let mut show_error = true;

		if status == 0 || status == 2 {
			println!("\rIgn: {id} {description}");
			// TODO: Add in support for apt configurations later
			// error_text is empty ||
			// _config->FindB("Acquire::Progress::Ignore::ShowErrorText", false) == false)

			// Do not show the error if it was simply ignored
			// show_error can be removed once configuration is added in
			show_error = false;
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

	/// Called periodically to provide the overall progress information
	///
	/// Draws the current progress.
	/// Each line has an overall percent meter and a per active item status
	/// meter along with an overall bandwidth and ETA indicator.
	fn pulse(
		&mut self,
		workers: Vec<apt::Worker>,
		percent: f32,
		total_bytes: u64,
		current_bytes: u64,
		current_cps: u64,
	) {
		if self.disable {
			return;
		}

		// Minus 1 for the cursor
		let term_width = self.screen_width();

		let mut string = String::new();
		let mut percent_str = format!("\r{percent:.0}%");
		let mut eta_str = String::new();

		// Set the ETA string if there is a rate of download
		if current_cps != 0 {
			let _ = write!(
				eta_str,
				" {} {}",
				// Current rate of download
				unit_str(current_cps, NumSys::Decimal),
				// ETA String
				time_str((total_bytes - current_bytes) / current_cps)
			);
		}

		for worker in workers {
			let mut work_string = String::new();
			work_string.push_str(" [");

			if !worker.is_current {
				if !worker.status.is_empty() {
					work_string.push_str(&worker.status);
					work_string.push(']');
				}
				continue;
			}

			if worker.id != 0 {
				let _ = write!(work_string, " {} ", worker.id);
			}

			work_string.push_str(&worker.short_desc);

			if !worker.active_subprocess.is_empty() {
				work_string.push(' ');
				work_string.push_str(&worker.active_subprocess);
			}

			work_string.push(' ');
			work_string.push_str(&unit_str(worker.current_size, NumSys::Decimal));

			if worker.total_size > 0 && !worker.complete {
				let _ = write!(
					work_string,
					"/{} {}",
					unit_str(worker.total_size, NumSys::Decimal),
					(worker.current_size * 100) / worker.total_size
				);
			}

			work_string.push(']');

			if (string.len() + work_string.len() + percent_str.len() + eta_str.len()) > term_width {
				break;
			}

			string.push_str(&work_string);
		}

		// Display at least something if there is no worker strings
		if string.is_empty() {
			string = " [Working]".to_string()
		}

		// Push the worker strings on the percent string
		percent_str.push_str(&string);

		// Fill the remaining space in the terminal if eta exists
		if !eta_str.is_empty() {
			let fill_size = percent_str.len() + eta_str.len();
			if fill_size < term_width {
				percent_str.push_str(&" ".repeat(term_width - fill_size))
			}
		}

		// Push the final eta to the end of the filled string
		percent_str.push_str(&eta_str);

		// Print and flush stdout
		print!("{percent_str}");
		stdout().flush().unwrap();

		if self.lastline > percent_str.len() {
			self.clear_last_line(term_width);
		}

		self.lastline = percent_str.len();
	}
}
