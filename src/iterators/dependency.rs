use std::cell::OnceCell;
use std::collections::HashMap;
use std::fmt;

use cxx::UniquePtr;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::raw::{DepIterator, VerIterator};
use crate::{Cache, Package, Version};

/// DepFlags defined in depcache.h
#[allow(non_upper_case_globals, non_snake_case)]
pub mod DepFlags {
	pub const DepNow: u8 = 1;
	pub const DepInstall: u8 = 2;
	pub const DepCVer: u8 = 4;
	pub const DepGNow: u8 = 8;
	pub const DepGInstall: u8 = 16;
	pub const DepGVer: u8 = 32;
}

#[cfg_attr(feature = "serde", derive(Serialize))]
/// The different types of Dependencies.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum DepType {
	Depends = 1,
	PreDepends = 2,
	Suggests = 3,
	Recommends = 4,
	Conflicts = 5,
	Replaces = 6,
	Obsoletes = 7,
	DpkgBreaks = 8,
	Enhances = 9,
}

impl From<u8> for DepType {
	fn from(value: u8) -> Self {
		match value {
			1 => DepType::Depends,
			2 => DepType::PreDepends,
			3 => DepType::Suggests,
			4 => DepType::Recommends,
			5 => DepType::Conflicts,
			6 => DepType::Replaces,
			7 => DepType::Obsoletes,
			8 => DepType::DpkgBreaks,
			9 => DepType::Enhances,
			_ => panic!("Dependency is malformed?"),
		}
	}
}

impl AsRef<str> for DepType {
	fn as_ref(&self) -> &str { self.to_str() }
}

impl DepType {
	pub fn to_str(&self) -> &'static str {
		match self {
			DepType::Depends => "Depends",
			DepType::PreDepends => "PreDepends",
			DepType::Suggests => "Suggests",
			DepType::Recommends => "Recommends",
			DepType::Conflicts => "Conflicts",
			DepType::Replaces => "Replaces",
			DepType::Obsoletes => "Obsoletes",
			DepType::DpkgBreaks => "Breaks",
			DepType::Enhances => "Enhances",
		}
	}
}

impl fmt::Display for DepType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.as_ref()) }
}

/// A struct representing a Base Dependency.
pub struct BaseDep<'a> {
	pub ptr: UniquePtr<DepIterator>,
	cache: &'a Cache,
	target: OnceCell<Package<'a>>,
	parent_ver: OnceCell<UniquePtr<VerIterator>>,
}

impl Clone for BaseDep<'_> {
	fn clone(&self) -> Self {
		Self {
			ptr: unsafe { self.ptr.unique() },
			cache: self.cache,
			target: self.target.clone(),
			parent_ver: unsafe { self.parent_ver().into() },
		}
	}
}

impl<'a> BaseDep<'a> {
	pub fn new(ptr: UniquePtr<DepIterator>, cache: &'a Cache) -> BaseDep<'a> {
		BaseDep {
			ptr,
			cache,
			target: OnceCell::new(),
			parent_ver: OnceCell::new(),
		}
	}

	/// This is the name of the dependency.
	pub fn name(&self) -> &str { self.target_package().name() }

	/// Return the target package.
	///
	/// For Reverse Dependencies this will actually return the parent package
	pub fn target_package(&self) -> &Package<'a> {
		self.target.get_or_init(|| {
			if self.is_reverse() {
				Package::new(self.cache, unsafe { self.parent_pkg() })
			} else {
				Package::new(self.cache, unsafe { self.target_pkg() })
			}
		})
	}

	/// The target version &str of the dependency if specified.
	pub fn version(&self) -> Option<&str> {
		if self.is_reverse() {
			Some(
				self.parent_ver
					.get_or_init(|| unsafe { self.parent_ver() })
					.version(),
			)
		} else {
			self.target_ver().ok()
		}
	}

	/// The Dependency Type. Depends, Recommends, etc.
	pub fn dep_type(&self) -> DepType { DepType::from(self.ptr.dep_type()) }

	/// Comparison type of the dependency version, if specified.
	pub fn comp_type(&self) -> Option<&str> { self.ptr.comp_type().ok() }

	// Iterate all Versions that are able to satisfy this dependency
	pub fn all_targets(&self) -> Vec<Version> {
		unsafe {
			self.ptr
				.all_targets()
				.iter()
				.map(|v| Version::new(v.unique(), self.cache))
				.collect()
		}
	}
}

impl fmt::Display for BaseDep<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if let (Some(comp), Some(version)) = (self.comp_type(), self.version()) {
			write!(f, "({} {comp} {version})", self.name())
		} else {
			write!(f, "({})", self.name())
		}
	}
}

impl fmt::Debug for BaseDep<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("BaseDep")
			.field("parent", unsafe { &self.parent_pkg().name() })
			.field("name", &self.name())
			.field("comp", &self.comp_type())
			.field("version", &self.version())
			.field("dep_type", &self.dep_type())
			.field("is_reverse", &self.is_reverse())
			.finish()
	}
}

/// A struct representing a single Dependency record.
///
/// This can contain multiple Base Dependencies that can
/// satisfy the same Dependency.
#[derive(fmt::Debug, Clone)]
pub struct Dependency<'a> {
	pub(crate) ptr: Vec<BaseDep<'a>>,
}

impl<'a> Dependency<'a> {
	/// Return the Dep Type of this group. Depends, Pre-Depends.
	pub fn dep_type(&self) -> DepType { self[0].dep_type() }

	/// Returns True if there are multiple dependencies that can satisfy this
	pub fn is_or(&self) -> bool { self.len() > 1 }

	/// Returns a reference to the first BaseDep
	pub fn first(&self) -> &BaseDep<'a> { &self[0] }
}

impl fmt::Display for Dependency<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut dep_str = String::new();

		for (i, base_dep) in self.iter().enumerate() {
			dep_str += &base_dep.to_string();
			if i + 1 != self.len() {
				dep_str += " | "
			}
		}

		write!(
			f,
			"{} {:?} {dep_str}",
			unsafe { self.first().parent_pkg().fullname(false) },
			self.dep_type(),
		)?;
		Ok(())
	}
}

pub fn create_depends_map(
	cache: &Cache,
	dep: Option<UniquePtr<DepIterator>>,
) -> HashMap<DepType, Vec<Dependency>> {
	let mut dependencies: HashMap<DepType, Vec<Dependency>> = HashMap::new();

	if let Some(mut dep) = dep {
		while !dep.end() {
			let mut or_deps = vec![];
			or_deps.push(BaseDep::new(unsafe { dep.unique() }, cache));

			// This means that more than one thing can satisfy a dependency.
			// For reverse dependencies we cannot get the or deps.
			// This can cause a segfault
			// See: https://gitlab.com/volian/rust-apt/-/merge_requests/36
			if dep.or_dep() && !dep.is_reverse() {
				loop {
					dep.pin_mut().raw_next();
					or_deps.push(BaseDep::new(unsafe { dep.unique() }, cache));
					// This is the last of the Or group
					if !dep.or_dep() {
						break;
					}
				}
			}

			let dep_type = DepType::from(dep.dep_type());

			// If the entry already exists in the map append it.
			if let Some(vec) = dependencies.get_mut(&dep_type) {
				vec.push(Dependency { ptr: or_deps })
			} else {
				// Doesn't exist so we create it
				dependencies.insert(dep_type, vec![Dependency { ptr: or_deps }]);
			}
			dep.pin_mut().raw_next();
		}
	}
	dependencies
}

#[cxx::bridge]
pub(crate) mod raw {
	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/package.h");

		type DepIterator;

		type PkgIterator = crate::raw::PkgIterator;
		type VerIterator = crate::raw::VerIterator;

		/// The Parent PkgIterator for this dependency
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn parent_pkg(self: &DepIterator) -> UniquePtr<PkgIterator>;

		/// The Parent VerIterator for this dependency
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn parent_ver(self: &DepIterator) -> UniquePtr<VerIterator>;

		// Dependency Declarations
		/// String representation of the dependency compare type
		/// "","<=",">=","<",">","=","!="
		///
		/// This returns Error for no compare type.
		pub fn comp_type(self: &DepIterator) -> Result<&str>;

		// Get the dependency type as a u8
		// You can use `DepType::from(raw_dep.dep_type())` to convert to enum.
		pub fn dep_type(self: &DepIterator) -> u8;

		/// Returns true if the dependency type is critical.
		///
		/// Depends, PreDepends, Conflicts, Obsoletes, Breaks
		/// will return [true].
		///
		/// Suggests, Recommends, Replaces and Enhances
		/// will return [false].
		#[cxx_name = "IsCritical"]
		pub fn is_critical(self: &DepIterator) -> bool;

		/// Return True if the dep is reverse, false if normal
		#[cxx_name = "Reverse"]
		pub fn is_reverse(self: &DepIterator) -> bool;

		pub fn target_ver(self: &DepIterator) -> Result<&str>;

		/// Return the Target Package for the dependency.
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn target_pkg(self: &DepIterator) -> UniquePtr<PkgIterator>;

		/// Returns a CxxVector of VerIterators.
		///
		/// # Safety
		///
		/// These can not be owned and will need to be Cloned with unique.
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn all_targets(self: &DepIterator) -> UniquePtr<CxxVector<VerIterator>>;

		/// Return true if this dep is Or'd with the next. The last dep in the
		/// or group will return False.
		pub fn or_dep(self: &DepIterator) -> bool;

		#[cxx_name = "Index"]
		pub fn index(self: &DepIterator) -> u64;
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
		unsafe fn unique(self: &DepIterator) -> UniquePtr<DepIterator>;
		pub fn raw_next(self: Pin<&mut DepIterator>);
		pub fn end(self: &DepIterator) -> bool;
	}
}
