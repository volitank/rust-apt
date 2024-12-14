//! Contains Progress struct for updating the package list.
use std::fmt::Write as _;
use std::io::{Write, stdout};
use std::os::fd::RawFd;
use std::pin::Pin;

use cxx::{ExternType, UniquePtr};

use crate::config::Config;
use crate::error::raw::pending_error;
use crate::raw::{AcqTextStatus, ItemDesc, ItemState, PkgAcquire, acquire_status};
use crate::util::{
	NumSys, get_apt_progress_string, terminal_height, terminal_width, time_str, unit_str,
};

/// Customize the output shown during file downloads.
pub trait DynAcquireProgress {
	/// Called on c++ to set the pulse interval.
	fn pulse_interval(&self) -> usize;

	/// Called when an item is confirmed to be up-to-date.
	fn hit(&mut self, item: &ItemDesc);

	/// Called when an Item has started to download
	fn fetch(&mut self, item: &ItemDesc);

	/// Called when an Item fails to download
	fn fail(&mut self, item: &ItemDesc);

	/// Called periodically to provide the overall progress information
	fn pulse(&mut self, status: &AcqTextStatus, owner: &PkgAcquire);

	/// Called when an item is successfully and completely fetched.
	fn done(&mut self, item: &ItemDesc);

	/// Called when progress has started
	fn start(&mut self);

	/// Called when progress has finished
	fn stop(&mut self, status: &AcqTextStatus);
}

/// Customize the output of operation progress on things like opening the cache.
pub trait DynOperationProgress {
	fn update(&mut self, operation: String, percent: f32);
	fn done(&mut self);
}

/// Customize the output of installation progress.
pub trait DynInstallProgress {
	fn status_changed(
		&mut self,
		pkgname: String,
		steps_done: u64,
		total_steps: u64,
		action: String,
	);
	fn error(&mut self, pkgname: String, steps_done: u64, total_steps: u64, error: String);
}

/// A struct aligning with `apt`'s AcquireStatus.
///
/// This struct takes a struct with impl AcquireProgress
/// It sets itself as the callback from C++ AcqTextStatus
/// which will then call the functions on this struct.
/// This struct will then forward those calls to your struct via
/// trait methods.
pub struct AcquireProgress<'a> {
	status: UniquePtr<AcqTextStatus>,
	inner: Box<dyn DynAcquireProgress + 'a>,
}

impl<'a> AcquireProgress<'a> {
	/// Create a new AcquireProgress Struct from a struct that implements
	/// AcquireProgress trait.
	pub fn new(inner: impl DynAcquireProgress + 'a) -> Self {
		Self {
			status: unsafe { acquire_status() },
			inner: Box::new(inner),
		}
	}

	/// Create a new AcquireProgress Struct with the default `apt`
	/// implementation.
	pub fn apt() -> Self { Self::new(AptAcquireProgress::new()) }

	/// Create a new AcquireProgress Struct that outputs nothing.
	pub fn quiet() -> Self { Self::new(AptAcquireProgress::disable()) }

	/// Sets AcquireProgress as the AcqTextStatus callback and
	/// returns a Pinned mutable reference to AcqTextStatus.
	pub fn mut_status(&mut self) -> Pin<&mut AcqTextStatus> {
		unsafe {
			// Create raw mutable pointer to ourself
			let raw_ptr = &mut *(self as *mut AcquireProgress);
			// Pin AcqTextStatus in place so it is not moved in memory
			// Segfault can occur if it is moved
			let mut status = self.status.pin_mut();

			// Set our raw pointer we created as the callback for C++ AcqTextStatus.
			// AcqTextStatus will then be fed into libapt who will call its methods
			// providing information. AcqTextStatus then uses this pointer to send that
			// information back to rust on this struct. This struct will then send it
			// through the trait methods on the `inner` object.
			status.as_mut().set_callback(raw_ptr);
			status
		}
	}

	/// Called on c++ to set the pulse interval.
	pub(crate) fn pulse_interval(&mut self) -> usize { self.inner.pulse_interval() }

	/// Called when an item is confirmed to be up-to-date.
	pub(crate) fn hit(&mut self, item: &ItemDesc) { self.inner.hit(item) }

	/// Called when an Item has started to download
	pub(crate) fn fetch(&mut self, item: &ItemDesc) { self.inner.fetch(item) }

	/// Called when an Item fails to download
	pub(crate) fn fail(&mut self, item: &ItemDesc) { self.inner.fail(item) }

	/// Called periodically to provide the overall progress information
	pub(crate) fn pulse(&mut self, owner: &PkgAcquire) { self.inner.pulse(&self.status, owner) }

	/// Called when progress has started
	pub(crate) fn start(&mut self) { self.inner.start() }

	/// Called when an item is successfully and completely fetched.
	pub(crate) fn done(&mut self, item: &ItemDesc) { self.inner.done(item) }

	/// Called when progress has finished
	pub(crate) fn stop(&mut self) { self.inner.stop(&self.status) }
}

impl Default for AcquireProgress<'_> {
	fn default() -> Self { Self::apt() }
}

/// Impl for sending AcquireProgress across the barrier.
unsafe impl ExternType for AcquireProgress<'_> {
	type Id = cxx::type_id!("AcquireProgress");
	type Kind = cxx::kind::Trivial;
}

/// Allows lengthy operations to communicate their progress.
///
/// The [`Default`] and only implementation of this is
/// [`self::OperationProgress::quiet`].
pub struct OperationProgress<'a> {
	inner: Box<dyn DynOperationProgress + 'a>,
}

impl<'a> OperationProgress<'a> {
	/// Create a new OpProgress Struct from a struct that implements
	/// AcquireProgress trait.
	pub fn new(inner: impl DynOperationProgress + 'static) -> Self {
		Self {
			inner: Box::new(inner),
		}
	}

	/// Returns a OperationProgress that outputs no data
	///
	/// Generally I have not found much use for displaying OpProgress
	pub fn quiet() -> Self { Self::new(NoOpProgress {}) }

	/// Called when an operation has been updated.
	fn update(&mut self, operation: String, percent: f32) { self.inner.update(operation, percent) }

	/// Called when an operation has finished.
	fn done(&mut self) { self.inner.done() }

	pub fn pin(&mut self) -> Pin<&mut OperationProgress<'a>> { Pin::new(self) }
}

impl Default for OperationProgress<'_> {
	fn default() -> Self { Self::quiet() }
}

/// Impl for sending AcquireProgress across the barrier.
unsafe impl ExternType for OperationProgress<'_> {
	type Id = cxx::type_id!("OperationProgress");
	type Kind = cxx::kind::Trivial;
}

/// Enum for displaying Progress of Package Installation.
///
/// The [`Default`] implementation mirrors apt's.
pub enum InstallProgress<'a> {
	Fancy(InstallProgressFancy<'a>),
	Fd(RawFd),
}

impl InstallProgress<'_> {
	/// Create a new OpProgress Struct from a struct that implements
	/// AcquireProgress trait.
	pub fn new(inner: impl DynInstallProgress + 'static) -> Self {
		Self::Fancy(InstallProgressFancy::new(inner))
	}

	/// Send dpkg status messages to an File Descriptor.
	/// This required more work to implement but is the most flexible.
	pub fn fd(fd: RawFd) -> Self { Self::Fd(fd) }

	/// Returns InstallProgress that mimics apt's fancy progress
	pub fn apt() -> Self { Self::new(AptInstallProgress::new()) }
}

impl Default for InstallProgress<'_> {
	fn default() -> Self { Self::apt() }
}

/// Struct for displaying Progress of Package Installation.
///
/// The [`Default`] implementation mirrors apt's.
pub struct InstallProgressFancy<'a> {
	inner: Box<dyn DynInstallProgress + 'a>,
}

impl<'a> InstallProgressFancy<'a> {
	/// Create a new OpProgress Struct from a struct that implements
	/// AcquireProgress trait.
	pub fn new(inner: impl DynInstallProgress + 'static) -> Self {
		Self {
			inner: Box::new(inner),
		}
	}

	/// Returns InstallProgress that mimics apt's fancy progress
	pub fn apt() -> Self { Self::new(AptInstallProgress::new()) }

	fn status_changed(
		&mut self,
		pkgname: String,
		steps_done: u64,
		total_steps: u64,
		action: String,
	) {
		self.inner
			.status_changed(pkgname, steps_done, total_steps, action)
	}

	fn error(&mut self, pkgname: String, steps_done: u64, total_steps: u64, error: String) {
		self.inner.error(pkgname, steps_done, total_steps, error)
	}

	pub fn pin(&mut self) -> Pin<&mut InstallProgressFancy<'a>> { Pin::new(self) }
}

impl Default for InstallProgressFancy<'_> {
	fn default() -> Self { Self::apt() }
}

/// Impl for sending InstallProgressFancy across the barrier.
unsafe impl ExternType for InstallProgressFancy<'_> {
	type Id = cxx::type_id!("InstallProgressFancy");
	type Kind = cxx::kind::Trivial;
}

/// Internal struct to pass into [`crate::Cache::resolve`]. The C++ library for
/// this wants a progress parameter for this, but it doesn't appear to be doing
/// anything. Furthermore, [the Python-APT implementation doesn't accept a
/// parameter for their dependency resolution functionality](https://apt-team.pages.debian.net/python-apt/library/apt_pkg.html#apt_pkg.ProblemResolver.resolve),
/// so we should be safe to remove it here.
struct NoOpProgress {}

impl DynOperationProgress for NoOpProgress {
	fn update(&mut self, _operation: String, _percent: f32) {}

	fn done(&mut self) {}
}

/// AptAcquireProgress is the default struct for the update method on the cache.
///
/// This struct mimics the output of `apt update`.
#[derive(Default, Debug)]
pub struct AptAcquireProgress {
	lastline: usize,
	pulse_interval: usize,
	disable: bool,
	config: Config,
}

impl AptAcquireProgress {
	/// Returns a new default progress instance.
	pub fn new() -> Self { Self::default() }

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

impl DynAcquireProgress for AptAcquireProgress {
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
	fn hit(&mut self, item: &ItemDesc) {
		if self.disable {
			return;
		}

		self.clear_last_line(terminal_width() - 1);

		println!("\rHit:{} {}", item.owner().id(), item.description());
	}

	/// Called when an Item has started to download
	///
	/// Prints out the short description and the expected size.
	fn fetch(&mut self, item: &ItemDesc) {
		if self.disable {
			return;
		}

		self.clear_last_line(terminal_width() - 1);

		let mut string = format!("\rGet:{} {}", item.owner().id(), item.description());

		let file_size = item.owner().file_size();
		if file_size != 0 {
			string.push_str(&format!(" [{}]", unit_str(file_size, NumSys::Decimal)));
		}

		println!("{string}");
	}

	/// Called when an item is successfully and completely fetched.
	///
	/// We don't print anything here to remain consistent with apt.
	fn done(&mut self, _item: &ItemDesc) {
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
	fn stop(&mut self, owner: &AcqTextStatus) {
		if self.disable {
			return;
		}

		self.clear_last_line(terminal_width() - 1);

		if pending_error() {
			return;
		}

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

	/// Called when an Item fails to download.
	///
	/// Print out the ErrorText for the Item.
	fn fail(&mut self, item: &ItemDesc) {
		if self.disable {
			return;
		}

		self.clear_last_line(terminal_width() - 1);

		let mut show_error = true;
		let error_text = item.owner().error_text();
		let desc = format!("{} {}", item.owner().id(), item.description());

		match item.owner().status() {
			ItemState::StatIdle | ItemState::StatDone => {
				println!("\rIgn: {desc}");
				let key = "Acquire::Progress::Ignore::ShowErrorText";
				if error_text.is_empty() || self.config.bool(key, false) {
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

	/// Called periodically to provide the overall progress information
	///
	/// Draws the current progress.
	/// Each line has an overall percent meter and a per active item status
	/// meter along with an overall bandwidth and ETA indicator.
	fn pulse(&mut self, status: &AcqTextStatus, owner: &PkgAcquire) {
		if self.disable {
			return;
		}

		// Minus 1 for the cursor
		let term_width = terminal_width() - 1;

		let mut string = String::new();
		let mut percent_str = format!("\r{:.0}%", status.percent());
		let mut eta_str = String::new();

		// Set the ETA string if there is a rate of download
		let current_cps = status.current_cps();
		if current_cps != 0 {
			let _ = write!(
				eta_str,
				" {} {}",
				// Current rate of download
				unit_str(current_cps, NumSys::Decimal),
				// ETA String
				time_str((status.total_bytes() - status.current_bytes()) / current_cps)
			);
		}

		for worker in owner.workers().iter() {
			let mut work_string = String::new();
			work_string.push_str(" [");

			let Ok(item) = worker.item() else {
				if !worker.status().is_empty() {
					work_string.push_str(&worker.status());
					work_string.push(']');
				}
				continue;
			};

			let id = item.owner().id();
			if id != 0 {
				let _ = write!(work_string, " {id} ");
			}
			work_string.push_str(&item.short_desc());

			let sub = item.owner().active_subprocess();
			if !sub.is_empty() {
				work_string.push(' ');
				work_string.push_str(&sub);
			}

			work_string.push(' ');
			work_string.push_str(&unit_str(worker.current_size(), NumSys::Decimal));

			if worker.total_size() > 0 && !item.owner().complete() {
				let _ = write!(
					work_string,
					"/{} {}%",
					unit_str(worker.total_size(), NumSys::Decimal),
					(worker.current_size() * 100) / worker.total_size()
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
	pub fn new() -> Self {
		Self {
			config: Config::new(),
		}
	}
}

impl Default for AptInstallProgress {
	fn default() -> Self { Self::new() }
}

impl DynInstallProgress for AptInstallProgress {
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
		print!("\x1b[{term_height};0f");
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

		print!("{bg_color}{fg_color}Progress: [{percent_str}%]{BG_COLOR_RESET}{FG_COLOR_RESET} ");

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

#[allow(clippy::needless_lifetimes)]
#[cxx::bridge]
pub(crate) mod raw {
	extern "Rust" {
		type AcquireProgress<'a>;
		type OperationProgress<'a>;
		type InstallProgressFancy<'a>;

		/// Called when an operation has been updated.
		fn update(self: &mut OperationProgress, operation: String, percent: f32);

		/// Called when an operation has finished.
		fn done(self: &mut OperationProgress);

		/// Called when the install status has changed.
		fn status_changed(
			self: &mut InstallProgressFancy,
			pkgname: String,
			steps_done: u64,
			total_steps: u64,
			action: String,
		);

		// TODO: What kind of errors can be returned here?
		// Research and update higher level structs as well
		// TODO: Create custom errors when we have better information
		fn error(
			self: &mut InstallProgressFancy,
			pkgname: String,
			steps_done: u64,
			total_steps: u64,
			error: String,
		);

		/// Called on c++ to set the pulse interval.
		fn pulse_interval(self: &mut AcquireProgress) -> usize;

		/// Called when an item is confirmed to be up-to-date.
		fn hit(self: &mut AcquireProgress, item: &ItemDesc);

		/// Called when an Item has started to download
		fn fetch(self: &mut AcquireProgress, item: &ItemDesc);

		/// Called when an Item fails to download
		fn fail(self: &mut AcquireProgress, item: &ItemDesc);

		/// Called periodically to provide the overall progress information
		fn pulse(self: &mut AcquireProgress, owner: &PkgAcquire);

		/// Called when an item is successfully and completely fetched.
		fn done(self: &mut AcquireProgress, item: &ItemDesc);

		/// Called when progress has started
		fn start(self: &mut AcquireProgress);

		/// Called when progress has finished
		fn stop(self: &mut AcquireProgress);
	}

	extern "C++" {
		type ItemDesc = crate::acquire::raw::ItemDesc;
		type PkgAcquire = crate::acquire::raw::PkgAcquire;
		include!("rust-apt/apt-pkg-c/types.h");
	}
}
