//! Contains Progress struct for updating the package list.
use std::fmt::Write as _;
use std::io::{stdout, Write};

use cxx::ExternType;

use crate::config::Config;
// use crate::config::Config;
use crate::util::{
	get_apt_progress_string, terminal_height, terminal_width, time_str, unit_str, NumSys,
};

pub type Worker = raw::Worker;

/// Trait you can impl on any struct to customize the output shown during file
/// downloads.
pub trait AcquireProgress {
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
		workers: Vec<Worker>,
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

/// Trait you can impl on any struct to customize the output of operation
/// progress on things like opening the cache.
pub trait OperationProgress {
	fn update(&mut self, operation: String, percent: f32);
	fn done(&mut self);
}

/// Internal struct to pass into [`self::Cache::resolve`]. The C++ library for
/// this wants a progress parameter for this, but it doesn't appear to be doing
/// anything. Furthermore, [the Python-APT implementation doesn't accept a
/// parameter for their dependency resolution funcionality](https://apt-team.pages.debian.net/python-apt/library/apt_pkg.html#apt_pkg.ProblemResolver.resolve),
/// so we should be safe to remove it here.
pub(crate) struct NoOpProgress {}

impl NoOpProgress {
	/// Return the AptAcquireProgress in a box
	/// To easily pass through for progress
	pub fn new_box() -> Box<dyn OperationProgress> { Box::new(NoOpProgress {}) }
}

impl OperationProgress for NoOpProgress {
	fn update(&mut self, _: String, _: f32) {}

	fn done(&mut self) {}
}

/// Trait you can impl on any struct to customize the output of installation
/// progress.
pub trait InstallProgress {
	fn status_changed(
		&mut self,
		pkgname: String,
		steps_done: u64,
		total_steps: u64,
		action: String,
	);
	fn error(&mut self, pkgname: String, steps_done: u64, total_steps: u64, error: String);
}

// TODO: Make better structs for pkgAcquire items, workers, owners.
/// AptAcquireProgress is the default struct for the update method on the cache.
///
/// This struct mimics the output of `apt update`.
#[derive(Default, Debug)]
pub struct AptAcquireProgress {
	lastline: usize,
	pulse_interval: usize,
	disable: bool,
}

impl AptAcquireProgress {
	/// Returns a new default progress instance.
	pub fn new() -> Self { Self::default() }

	/// Return the AptAcquireProgress in a box
	/// To easily pass through for progress
	pub fn new_box() -> Box<dyn AcquireProgress> { Box::new(Self::new()) }

	/// Returns a disabled progress instance. No output will be shown.
	pub fn disable() -> Self {
		AptAcquireProgress {
			disable: true,
			..Default::default()
		}
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

impl AcquireProgress for AptAcquireProgress {
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

		self.clear_last_line(terminal_width() - 1);

		println!("\rHit:{} {}", id, description);
	}

	/// Called when an Item has started to download
	///
	/// Prints out the short description and the expected size.
	fn fetch(&mut self, id: u32, description: String, file_size: u64) {
		if self.disable {
			return;
		}

		self.clear_last_line(terminal_width() - 1);

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
		// self.clear_last_line(terminal_width() - 1);

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

		self.clear_last_line(terminal_width() - 1);

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

		self.clear_last_line(terminal_width() - 1);

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
		workers: Vec<Worker>,
		percent: f32,
		total_bytes: u64,
		current_bytes: u64,
		current_cps: u64,
	) {
		if self.disable {
			return;
		}

		// Minus 1 for the cursor
		let term_width = terminal_width() - 1;

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

/// Default struct to handle the output of a transaction.
pub struct AptInstallProgress {
	config: Config,
}

impl AptInstallProgress {
	#[allow(dead_code)]
	pub fn new() -> Self {
		Self {
			config: Config::new(),
		}
	}

	/// Return the AptInstallProgress in a box
	/// To easily pass through to do_install
	pub fn new_box() -> Box<dyn InstallProgress> { Box::new(Self::new()) }
}

impl Default for AptInstallProgress {
	fn default() -> Self { Self::new() }
}

impl InstallProgress for AptInstallProgress {
	fn status_changed(
		&mut self,
		_pkgname: String,
		steps_done: u64,
		total_steps: u64,
		_action: String,
	) {
		// Get the terminal's width and height.
		let term_height = terminal_height();
		let term_width = terminal_width();

		// Save the current cursor position.
		print!("\x1b7");

		// Go to the progress reporting line.
		print!("\x1b[{};0f", term_height);
		std::io::stdout().flush().unwrap();

		// Convert the float to a percentage string.
		let percent = steps_done as f32 / total_steps as f32;
		let mut percent_str = (percent * 100.0).round().to_string();

		let percent_padding = match percent_str.len() {
			1 => "  ",
			2 => " ",
			3 => "",
			_ => unreachable!(),
		};

		percent_str = percent_padding.to_owned() + &percent_str;

		// Get colors for progress reporting.
		// NOTE: The APT implementation confusingly has 'Progress-fg' for 'bg_color',
		// and the same the other way around.
		let bg_color = self
			.config
			.find("Dpkg::Progress-Fancy::Progress-fg", "\x1b[42m");
		let fg_color = self
			.config
			.find("Dpkg::Progress-Fancy::Progress-bg", "\x1b[30m");
		const BG_COLOR_RESET: &str = "\x1b[49m";
		const FG_COLOR_RESET: &str = "\x1b[39m";

		print!(
			"{}{}Progress: [{}%]{}{} ",
			bg_color, fg_color, percent_str, BG_COLOR_RESET, FG_COLOR_RESET
		);

		// The length of "Progress: [100%] ".
		const PROGRESS_STR_LEN: usize = 17;

		// Print the progress bar.
		// We should safely be able to convert the `usize`.try_into() into the `u32`
		// needed by `get_apt_progress_string`, as usize ints only take up 8 bytes on a
		// 64-bit processor.
		print!(
			"{}",
			get_apt_progress_string(percent, (term_width - PROGRESS_STR_LEN).try_into().unwrap())
		);
		std::io::stdout().flush().unwrap();

		// If this is the last change, remove the progress reporting bar.
		// if steps_done == total_steps {
		// print!("{}", " ".repeat(term_width));
		// print!("\x1b[0;{}r", term_height);
		// }
		// Finally, go back to the previous cursor position.
		print!("\x1b8");
		std::io::stdout().flush().unwrap();
	}

	// TODO: Need to figure out when to use this.
	fn error(&mut self, _pkgname: String, _steps_done: u64, _total_steps: u64, _error: String) {}
}

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {

	/// A simple representation of an Acquire worker.
	///
	/// TODO: Make this better.
	struct Worker {
		is_current: bool,
		status: String,
		id: u64,
		short_desc: String,
		active_subprocess: String,
		current_size: u64,
		total_size: u64,
		complete: bool,
	}

	extern "Rust" {
		/// Called on c++ to set the pulse interval.
		fn pulse_interval(progress: &mut DynAcquireProgress) -> usize;

		/// Called when an item is confirmed to be up-to-date.
		fn hit(progress: &mut DynAcquireProgress, id: u32, description: String);

		/// Called when an Item has started to download
		fn fetch(progress: &mut DynAcquireProgress, id: u32, description: String, file_size: u64);

		/// Called when an Item fails to download
		fn fail(
			progress: &mut DynAcquireProgress,
			id: u32,
			description: String,
			status: u32,
			error_text: String,
		);

		/// Called periodically to provide the overall progress information
		fn pulse(
			progress: &mut DynAcquireProgress,
			workers: Vec<Worker>,
			percent: f32,
			total_bytes: u64,
			current_bytes: u64,
			current_cps: u64,
		);

		/// Called when an item is successfully and completely fetched.
		fn done(progress: &mut DynAcquireProgress);

		/// Called when progress has started
		fn start(progress: &mut DynAcquireProgress);

		/// Called when progress has finished
		fn stop(
			progress: &mut DynAcquireProgress,
			fetched_bytes: u64,
			elapsed_time: u64,
			current_cps: u64,
			pending_errors: bool,
		);

		/// Called when an operation has been updated.
		fn op_update(progress: &mut DynOperationProgress, operation: String, percent: f32);

		/// Called when an operation has finished.
		fn op_done(progress: &mut DynOperationProgress);

		///
		fn inst_status_changed(
			progress: &mut DynInstallProgress,
			pkgname: String,
			steps_done: u64,
			total_steps: u64,
			action: String,
		);

		// TODO: What kind of errors can be returned here?
		// Research and update higher level structs as well
		// TODO: Create custom errors when we have better information
		fn inst_error(
			progress: &mut DynInstallProgress,
			pkgname: String,
			steps_done: u64,
			total_steps: u64,
			error: String,
		);
	}

	unsafe extern "C++" {
		type DynAcquireProgress = Box<dyn crate::raw::progress::AcquireProgress>;
		type DynOperationProgress = Box<dyn crate::raw::progress::OperationProgress>;
		type DynInstallProgress = Box<dyn crate::raw::progress::InstallProgress>;

		include!("rust-apt/apt-pkg-c/progress.h");
	}
}

/// Impl for sending AcquireProgress across the barrier.
unsafe impl ExternType for Box<dyn AcquireProgress> {
	type Id = cxx::type_id!("DynAcquireProgress");
	type Kind = cxx::kind::Trivial;
}

/// Impl for sending OperationProgress across the barrier.
/// TODO: Needs to be reviewed in GitLab MR, because I've got just about zero
/// clue what I'm doing.
unsafe impl ExternType for Box<dyn OperationProgress> {
	type Id = cxx::type_id!("DynOperationProgress");
	type Kind = cxx::kind::Trivial;
}

/// Impl for sending InstallProgress across the barrier.
/// TODO: Needs to be reviewed in GitLab MR, because I've got just about zero
/// clue what I'm doing.
unsafe impl ExternType for Box<dyn InstallProgress> {
	type Id = cxx::type_id!("DynInstallProgress");
	type Kind = cxx::kind::Trivial;
}

// Begin AcquireProgress trait functions
// These must be defined outside the cxx bridge but in the same file

/// Called on c++ to set the pulse interval.
fn pulse_interval(progress: &mut Box<dyn AcquireProgress>) -> usize {
	(**progress).pulse_interval()
}

/// Called when an item is confirmed to be up-to-date.
fn hit(progress: &mut Box<dyn AcquireProgress>, id: u32, description: String) {
	(**progress).hit(id, description)
}

/// Called when an Item has started to download
fn fetch(progress: &mut Box<dyn AcquireProgress>, id: u32, description: String, file_size: u64) {
	(**progress).fetch(id, description, file_size)
}

/// Called when an Item fails to download
fn fail(
	progress: &mut Box<dyn AcquireProgress>,
	id: u32,
	description: String,
	status: u32,
	error_text: String,
) {
	(**progress).fail(id, description, status, error_text)
}

/// Called periodically to provide the overall progress information
fn pulse(
	progress: &mut Box<dyn AcquireProgress>,
	workers: Vec<Worker>,
	percent: f32,
	total_bytes: u64,
	current_bytes: u64,
	current_cps: u64,
) {
	(**progress).pulse(workers, percent, total_bytes, current_bytes, current_cps)
}

/// Called when an item is successfully and completely fetched.
fn done(progress: &mut Box<dyn AcquireProgress>) { (**progress).done() }

/// Called when progress has started
fn start(progress: &mut Box<dyn AcquireProgress>) { (**progress).start() }

/// Called when progress has finished
fn stop(
	progress: &mut Box<dyn AcquireProgress>,
	fetched_bytes: u64,
	elapsed_time: u64,
	current_cps: u64,
	pending_errors: bool,
) {
	(**progress).stop(fetched_bytes, elapsed_time, current_cps, pending_errors)
}

// End AcquireProgress trait functions

// Begin OperationProgress trait functions
// These must be defined outside the cxx bridge but in the same file

/// Called when an operation has been updated.
fn op_update(progress: &mut Box<dyn OperationProgress>, operation: String, percent: f32) {
	(**progress).update(operation, percent)
}

/// Called when an operation has finished.
fn op_done(progress: &mut Box<dyn OperationProgress>) { (**progress).done() }

// End OperationProgress trait functions

// Begin InstallProgress trait functions

fn inst_status_changed(
	progress: &mut Box<dyn InstallProgress>,
	pkgname: String,
	steps_done: u64,
	total_steps: u64,
	action: String,
) {
	(**progress).status_changed(pkgname, steps_done, total_steps, action)
}

fn inst_error(
	progress: &mut Box<dyn InstallProgress>,
	pkgname: String,
	steps_done: u64,
	total_steps: u64,
	error: String,
) {
	(**progress).error(pkgname, steps_done, total_steps, error)
}

// End InstallProgress trait functions
