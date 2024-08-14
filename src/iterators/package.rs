use std::cell::OnceCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

use cxx::UniquePtr;

use crate::raw::{IntoRawIter, PkgIterator};
use crate::{create_depends_map, util, Cache, DepType, Dependency, Provider, Version};
/// The state that the user wishes the package to be in.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum PkgSelectedState {
	Unknown = 0,
	Install = 1,
	Hold = 2,
	DeInstall = 3,
	Purge = 4,
}

impl From<u8> for PkgSelectedState {
	fn from(value: u8) -> Self {
		match value {
			0 => PkgSelectedState::Unknown,
			1 => PkgSelectedState::Install,
			2 => PkgSelectedState::Hold,
			3 => PkgSelectedState::DeInstall,
			4 => PkgSelectedState::Purge,
			_ => panic!("PkgSelectedState is malformed?"),
		}
	}
}

/// Installation state of the package
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum PkgInstState {
	Ok = 0,
	ReInstReq = 1,
	HoldInst = 2,
	HoldReInstReq = 3,
}

impl From<u8> for PkgInstState {
	fn from(value: u8) -> Self {
		match value {
			0 => PkgInstState::Ok,
			1 => PkgInstState::ReInstReq,
			2 => PkgInstState::HoldInst,
			3 => PkgInstState::HoldReInstReq,
			_ => panic!("PkgInstState is malformed?"),
		}
	}
}

/// The current state of a Package.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum PkgCurrentState {
	NotInstalled = 0,
	UnPacked = 1,
	HalfConfigured = 2,
	HalfInstalled = 4,
	ConfigFiles = 5,
	Installed = 6,
	TriggersAwaited = 7,
	TriggersPending = 8,
}

impl From<u8> for PkgCurrentState {
	fn from(value: u8) -> Self {
		match value {
			0 => PkgCurrentState::NotInstalled,
			1 => PkgCurrentState::UnPacked,
			2 => PkgCurrentState::HalfConfigured,
			4 => PkgCurrentState::HalfInstalled,
			5 => PkgCurrentState::ConfigFiles,
			6 => PkgCurrentState::Installed,
			7 => PkgCurrentState::TriggersAwaited,
			8 => PkgCurrentState::TriggersPending,
			_ => panic!("PkgCurrentState is malformed?"),
		}
	}
}

#[derive(Debug)]
pub enum Marked {
	NewInstall,
	Install,
	ReInstall,
	Remove,
	Purge,
	Keep,
	Upgrade,
	Downgrade,
	Held,
	None,
}

/// A single unique libapt package.
pub struct Package<'a> {
	pub(crate) ptr: UniquePtr<PkgIterator>,
	pub(crate) cache: &'a Cache,
	rdepends_map: OnceCell<HashMap<DepType, Vec<Dependency<'a>>>>,
}

impl<'a> Package<'a> {
	pub fn new(cache: &'a Cache, ptr: UniquePtr<PkgIterator>) -> Package<'a> {
		Package {
			ptr,
			cache,
			rdepends_map: OnceCell::new(),
		}
	}

	/// Returns a Reverse Dependency Map of the package
	///
	/// Dependencies are in a `Vec<Dependency>`
	///
	/// The Dependency struct represents an Or Group of dependencies.
	///
	/// For example where we use the [`crate::DepType::Depends`] key:
	///
	/// ```
	/// use rust_apt::{new_cache, DepType};
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("apt").unwrap();
	/// for dep in pkg.rdepends().get(&DepType::Depends).unwrap() {
	///    if dep.is_or() {
	///        for base_dep in dep.iter() {
	///            println!("{}", base_dep.name())
	///        }
	///    } else {
	///        // is_or is false so there is only one BaseDep
	///        println!("{}", dep.first().name())
	///    }
	/// }
	/// ```
	pub fn rdepends(&self) -> &HashMap<DepType, Vec<Dependency<'a>>> {
		self.rdepends_map.get_or_init(|| {
			create_depends_map(self.cache, unsafe { self.ptr.rdepends().make_safe() })
		})
	}

	/// Return either a Version or None
	///
	/// # Example:
	/// ```
	/// use rust_apt::new_cache;
	///
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("apt").unwrap();
	///
	/// pkg.get_version("2.4.7");
	/// ```
	pub fn get_version(&self, version_str: &str) -> Option<Version<'a>> {
		for ver in unsafe { self.ptr.versions().raw_iter() } {
			if version_str == ver.version() {
				return Some(Version::new(ver, self.cache));
			}
		}
		None
	}

	/// True if the Package is installed.
	pub fn is_installed(&self) -> bool { unsafe { !self.current_version().end() } }

	/// True if the package has versions.
	///
	/// If a package has no versions it is considered virtual.
	pub fn has_versions(&self) -> bool { unsafe { !self.ptr.versions().end() } }

	/// True if the package provides any other packages.
	pub fn has_provides(&self) -> bool { unsafe { !self.ptr.provides().end() } }

	/// The installed state of this package.
	pub fn inst_state(&self) -> PkgInstState { PkgInstState::from(self.ptr.inst_state()) }

	/// The selected state of this package.
	pub fn selected_state(&self) -> PkgSelectedState {
		PkgSelectedState::from(self.ptr.selected_state())
	}

	/// The current state of this package.
	pub fn current_state(&self) -> PkgCurrentState {
		PkgCurrentState::from(self.ptr.current_state())
	}

	/// Returns the version object of the installed version.
	///
	/// If there isn't an installed version, returns None
	pub fn installed(&self) -> Option<Version<'a>> {
		Some(Version::new(
			unsafe { self.current_version().make_safe() }?,
			self.cache,
		))
	}

	/// Returns the version object of the candidate.
	///
	/// If there isn't a candidate, returns None
	pub fn candidate(&self) -> Option<Version<'a>> {
		Some(Version::new(
			unsafe { self.cache.depcache().candidate_version(self).make_safe()? },
			self.cache,
		))
	}

	/// Returns the install version if it exists.
	///
	/// # This differs from [`crate::Package::installed`] in the
	/// # following ways:
	///
	/// * If a version is marked for install this will return the version to be
	///   installed.
	/// * If an installed package is marked for removal, this will return
	///   [`None`].
	pub fn install_version(&self) -> Option<Version<'a>> {
		// Cxx error here just indicates that the Version doesn't exist
		Some(Version::new(
			unsafe { self.cache.depcache().install_version(self).make_safe()? },
			self.cache,
		))
	}

	/// Returns a version list
	/// starting with the newest and ending with the oldest.
	pub fn versions(&self) -> impl Iterator<Item = Version<'a>> {
		unsafe { self.ptr.versions() }
			.raw_iter()
			.map(|ver| Version::new(ver, self.cache))
	}

	/// Returns a list of providers
	pub fn provides(&self) -> impl Iterator<Item = Provider<'a>> {
		unsafe { self.ptr.provides() }
			.raw_iter()
			.map(|p| Provider::new(p, self.cache))
	}

	/// Check if the package is upgradable.
	pub fn is_upgradable(&self) -> bool {
		self.is_installed() && self.cache.depcache().is_upgradable(self)
	}

	/// Check if the package is auto installed. (Not installed by the user)
	pub fn is_auto_installed(&self) -> bool { self.cache.depcache().is_auto_installed(self) }

	/// Check if the package is auto removable
	pub fn is_auto_removable(&self) -> bool {
		(self.is_installed() || self.marked_install()) && self.cache.depcache().is_garbage(self)
	}

	pub fn marked(&self) -> Marked {
		// Accessors that do not check `Mode` internally must come first

		// Held is also marked keep. It needs to come before keep.
		if self.marked_held() {
			return Marked::Held;
		}

		if self.marked_keep() {
			return Marked::Keep;
		}

		// Upgrade, NewInstall, Reinstall and Downgrade are marked Install.
		// They need to come before Install.
		if self.marked_reinstall() {
			return Marked::ReInstall;
		}

		if self.marked_upgrade() && self.is_installed() {
			return Marked::Upgrade;
		}

		if self.marked_new_install() {
			return Marked::NewInstall;
		}

		if self.marked_downgrade() {
			return Marked::Downgrade;
		}

		if self.marked_install() {
			return Marked::Install;
		}

		// Purge is also marked delete. Needs to come first.
		if self.marked_purge() {
			return Marked::Purge;
		}

		if self.marked_delete() {
			return Marked::Remove;
		}

		Marked::None
	}

	/// Check if the package is now broken
	pub fn is_now_broken(&self) -> bool { self.cache.depcache().is_now_broken(self) }

	/// Check if the package package installed is broken
	pub fn is_inst_broken(&self) -> bool { self.cache.depcache().is_inst_broken(self) }

	/// Check if the package is marked NewInstall
	pub fn marked_new_install(&self) -> bool { self.cache.depcache().marked_new_install(self) }

	/// Check if the package is marked install
	pub fn marked_install(&self) -> bool { self.cache.depcache().marked_install(self) }

	/// Check if the package is marked upgrade
	pub fn marked_upgrade(&self) -> bool { self.cache.depcache().marked_upgrade(self) }

	/// Check if the package is marked purge
	pub fn marked_purge(&self) -> bool { self.cache.depcache().marked_purge(self) }

	/// Check if the package is marked delete
	pub fn marked_delete(&self) -> bool { self.cache.depcache().marked_delete(self) }

	/// Check if the package is marked held
	pub fn marked_held(&self) -> bool { self.cache.depcache().marked_held(self) }

	/// Check if the package is marked keep
	pub fn marked_keep(&self) -> bool { self.cache.depcache().marked_keep(self) }

	/// Check if the package is marked downgrade
	pub fn marked_downgrade(&self) -> bool { self.cache.depcache().marked_downgrade(self) }

	/// Check if the package is marked reinstall
	pub fn marked_reinstall(&self) -> bool { self.cache.depcache().marked_reinstall(self) }

	/// # Mark a package as automatically installed.
	///
	/// ## mark_auto:
	///   * [true] = Mark the package as automatically installed.
	///   * [false] = Mark the package as manually installed.
	pub fn mark_auto(&self, mark_auto: bool) -> bool {
		self.cache.depcache().mark_auto(self, mark_auto);
		// Convert to a bool to remain consistent with other mark functions.
		true
	}

	/// # Mark a package for keep.
	///
	/// ## Returns:
	///   * [true] if the mark was successful
	///   * [false] if the mark was unsuccessful
	///
	/// This means that the package will not be changed from its current
	/// version. This will not stop a reinstall, but will stop removal, upgrades
	/// and downgrades
	///
	/// We don't believe that there is any reason to unmark packages for keep.
	/// If someone has a reason, and would like it implemented, please put in a
	/// feature request.
	pub fn mark_keep(&self) -> bool { self.cache.depcache().mark_keep(self) }

	/// # Mark a package for removal.
	///
	/// ## Returns:
	///   * [true] if the mark was successful
	///   * [false] if the mark was unsuccessful
	///
	/// ## purge:
	///   * [true] = Configuration files will be removed along with the package.
	///   * [false] = Only the package will be removed.
	pub fn mark_delete(&self, purge: bool) -> bool {
		self.cache.depcache().mark_delete(self, purge)
	}

	/// # Mark a package for installation.
	///
	/// ## auto_inst:
	///   * [true] = Additionally mark the dependencies for this package.
	///   * [false] = Mark only this package.
	///
	/// ## from_user:
	///   * [true] = The package will be marked manually installed.
	///   * [false] = The package will be unmarked automatically installed.
	///
	/// ## Returns:
	///   * [true] if the mark was successful
	///   * [false] if the mark was unsuccessful
	///
	/// If a package is already installed, at the latest version,
	/// and you mark that package for install you will get true,
	/// but the package will not be altered.
	/// `pkg.marked_install()` will be false
	pub fn mark_install(&self, auto_inst: bool, from_user: bool) -> bool {
		self.cache
			.depcache()
			.mark_install(self, auto_inst, from_user)
	}

	/// # Mark a package for reinstallation.
	///
	/// ## Returns:
	///   * [true] if the mark was successful
	///   * [false] if the mark was unsuccessful
	///
	/// ## reinstall:
	///   * [true] = The package will be marked for reinstall.
	///   * [false] = The package will be unmarked for reinstall.
	pub fn mark_reinstall(&self, reinstall: bool) -> bool {
		self.cache.depcache().mark_reinstall(self, reinstall);
		// Convert to a bool to remain consistent with other mark functions/
		true
	}

	/// Protect a package's state
	/// for when [`crate::cache::Cache::resolve`] is called.
	pub fn protect(&self) { self.cache.resolver().protect(self) }

	pub fn changelog_uri(&self) -> Option<String> {
		let cand = self.candidate()?;

		let src_pkg = cand.source_name();
		let mut src_ver = cand.source_version().to_string();
		let mut section = cand.section().ok()?.to_string();

		if let Ok(src_records) = self.cache.source_records() {
			while let Some(record) = src_records.lookup(src_pkg.to_string(), false) {
				let record_version = record.version();

				match util::cmp_versions(&record_version, &src_ver) {
					Ordering::Equal | Ordering::Greater => {
						src_ver = record_version;
						section = record.section();
						break;
					},
					_ => {},
				}
			}
		}

		let base_url = match cand.package_files().next()?.origin()? {
			"Ubuntu" => "http://changelogs.ubuntu.com/changelogs/pool",
			"Debian" => "http://packages.debian.org/changelogs/pool",
			_ => return None,
		};

		let prefix = if src_pkg.starts_with("lib") {
			format!("lib{}", src_pkg.chars().nth(3)?)
		} else {
			src_pkg.chars().next()?.to_string()
		};

		Some(format!(
			"{base_url}/{}/{prefix}/{src_pkg}/{src_pkg}_{}/changelog",
			if section.contains('/') { section.split('/').nth(0)? } else { "main" },
			// Strip epoch
			if let Some(split) = src_ver.split_once(':') { split.1 } else { &src_ver }
		))
	}
}

impl<'a> fmt::Display for Package<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.name())?;
		Ok(())
	}
}

impl<'a> fmt::Debug for Package<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let versions: Vec<Version> = self.versions().collect();
		f.debug_struct("Package")
			.field("name", &self.name())
			.field("arch", &self.arch())
			.field("virtual", &versions.is_empty())
			.field("versions", &versions)
			.finish_non_exhaustive()
	}
}

#[cxx::bridge]
pub(crate) mod raw {
	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/package.h");

		type PkgIterator;
		type VerIterator = crate::iterators::VerIterator;
		type PrvIterator = crate::iterators::PrvIterator;
		type DepIterator = crate::iterators::DepIterator;

		/// Get the name of the package without the architecture.
		pub fn name(self: &PkgIterator) -> &str;

		/// Get the architecture of a package.
		pub fn arch(self: &PkgIterator) -> &str;

		/// Get the fullname of the package.
		///
		/// Pretty is a bool that will omit the native arch.
		pub fn fullname(self: &PkgIterator, pretty: bool) -> String;

		/// Get the current state of a package.
		pub fn current_state(self: &PkgIterator) -> u8;

		/// Get the installed state of a package.
		pub fn inst_state(self: &PkgIterator) -> u8;

		/// Get the selected state of a package.
		pub fn selected_state(self: &PkgIterator) -> u8;

		/// True if the package is essential.
		pub fn is_essential(self: &PkgIterator) -> bool;

		/// Get a pointer the the currently installed version.
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn current_version(self: &PkgIterator) -> UniquePtr<VerIterator>;

		/// Get a pointer to the beginning of the VerIterator.
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn versions(self: &PkgIterator) -> UniquePtr<VerIterator>;

		/// Get the providers of this package.
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn provides(self: &PkgIterator) -> UniquePtr<PrvIterator>;

		/// Get the reverse dependencies of this package
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn rdepends(self: &PkgIterator) -> UniquePtr<DepIterator>;

		#[cxx_name = "Index"]
		pub fn index(self: &PkgIterator) -> u64;
		/// Clone the pointer.
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn unique(self: &PkgIterator) -> UniquePtr<PkgIterator>;
		pub fn raw_next(self: Pin<&mut PkgIterator>);
		pub fn end(self: &PkgIterator) -> bool;
	}
}
