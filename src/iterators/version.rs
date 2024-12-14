use std::cell::OnceCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

use cxx::UniquePtr;

use crate::raw::{IntoRawIter, VerIterator};
use crate::util::cmp_versions;
use crate::{
	Cache, DepType, Dependency, Package, PackageFile, PackageRecords, Provider, VersionFile,
	create_depends_map,
};

/// Represents a single Version of a package.
pub struct Version<'a> {
	pub(crate) ptr: UniquePtr<VerIterator>,
	cache: &'a Cache,
	depends_map: OnceCell<HashMap<DepType, Vec<Dependency<'a>>>>,
}

impl<'a> Clone for Version<'a> {
	fn clone(&self) -> Self {
		Self {
			ptr: unsafe { self.ptr.unique() },
			cache: self.cache,
			depends_map: self.depends_map.clone(),
		}
	}
}

impl<'a> Version<'a> {
	pub fn new(ptr: UniquePtr<VerIterator>, cache: &'a Cache) -> Version<'a> {
		Version {
			ptr,
			cache,
			depends_map: OnceCell::new(),
		}
	}

	/// Returns a list of providers
	pub fn provides(&self) -> impl Iterator<Item = Provider<'a>> {
		unsafe { self.ptr.provides() }
			.raw_iter()
			.map(|p| Provider::new(p, self.cache))
	}

	pub fn version_files(&self) -> impl Iterator<Item = VersionFile<'a>> {
		unsafe { self.ptr.version_files() }
			.raw_iter()
			.map(|v| VersionFile::new(v, self.cache))
	}

	/// Returns an iterator of PackageFiles (Origins) for the version
	pub fn package_files(&self) -> impl Iterator<Item = PackageFile<'a>> {
		self.version_files().map(|v| v.package_file())
	}

	/// Return the version's parent package.
	pub fn parent(&self) -> Package<'a> { Package::new(self.cache, unsafe { self.parent_pkg() }) }

	/// Returns a reference to the Dependency Map owned by the Version
	///
	/// Dependencies are in a `Vec<Dependency>`
	///
	/// The Dependency struct represents an Or Group of dependencies.
	/// The base deps are located in `Dependency.base_deps`
	///
	/// For example where we use the `DepType::Depends` key:
	///
	/// ```
	/// use rust_apt::{new_cache, DepType};
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("apt").unwrap();
	/// let version = pkg.candidate().unwrap();
	/// for dep in version.depends_map().get(&DepType::Depends).unwrap() {
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
	pub fn depends_map(&self) -> &HashMap<DepType, Vec<Dependency<'a>>> {
		self.depends_map
			.get_or_init(|| create_depends_map(self.cache, unsafe { self.depends().make_safe() }))
	}

	/// Returns a reference Vector, if it exists, for the given key.
	///
	/// See the doc for `depends_map()` for more information.
	pub fn get_depends(&self, key: &DepType) -> Option<&Vec<Dependency<'a>>> {
		self.depends_map().get(key)
	}

	/// Returns a Reference Vector, if it exists, for "Enhances".
	pub fn enhances(&self) -> Option<&Vec<Dependency<'a>>> { self.get_depends(&DepType::Enhances) }

	/// Returns a Reference Vector, if it exists,
	/// for "Depends" and "PreDepends".
	pub fn dependencies(&self) -> Option<Vec<&Dependency<'a>>> {
		let mut ret_vec: Vec<&Dependency> = Vec::new();

		for dep_type in [DepType::Depends, DepType::PreDepends] {
			if let Some(dep_list) = self.get_depends(&dep_type) {
				for dep in dep_list {
					ret_vec.push(dep)
				}
			}
		}

		if ret_vec.is_empty() {
			return None;
		}
		Some(ret_vec)
	}

	/// Returns a Reference Vector, if it exists, for "Recommends".
	pub fn recommends(&self) -> Option<&Vec<Dependency<'a>>> {
		self.get_depends(&DepType::Recommends)
	}

	/// Returns a Reference Vector, if it exists, for "suggests".
	pub fn suggests(&self) -> Option<&Vec<Dependency<'a>>> { self.get_depends(&DepType::Suggests) }

	/// Move the PkgRecords into the correct place for the Description
	fn desc_lookup(&self) -> Option<&PackageRecords> {
		let desc = unsafe { self.translated_desc().make_safe()? };
		Some(self.cache.records().desc_lookup(&desc))
	}

	/// Get the translated long description
	pub fn description(&self) -> Option<String> { self.desc_lookup()?.long_desc() }

	/// Get the translated short description
	pub fn summary(&self) -> Option<String> { self.desc_lookup()?.short_desc() }

	/// Get data from the specified record field
	///
	/// # Returns:
	///   * Some String or None if the field doesn't exist.
	///
	/// # Example:
	/// ```
	/// use rust_apt::new_cache;
	/// use rust_apt::records::RecordField;
	///
	/// let cache = new_cache!().unwrap();
	/// let pkg = cache.get("apt").unwrap();
	/// let cand = pkg.candidate().unwrap();
	///
	/// println!("{}", cand.get_record(RecordField::Maintainer).unwrap());
	/// // Or alternatively you can just pass any string
	/// println!("{}", cand.get_record("Description-md5").unwrap());
	/// ```
	pub fn get_record<T: ToString + ?Sized>(&self, field: &T) -> Option<String> {
		self.version_files()
			.next()?
			.lookup()
			.get_field(field.to_string())
	}

	/// Get the hash specified. If there isn't one returns None
	/// `version.hash("md5sum")`
	pub fn hash<T: ToString + ?Sized>(&self, hash_type: &T) -> Option<String> {
		self.version_files()
			.next()?
			.lookup()
			.hash_find(hash_type.to_string())
	}

	/// Get the sha256 hash. If there isn't one returns None
	/// This is equivalent to `version.hash("sha256")`
	pub fn sha256(&self) -> Option<String> { self.hash("sha256") }

	/// Get the sha512 hash. If there isn't one returns None
	/// This is equivalent to `version.hash("sha512")`
	pub fn sha512(&self) -> Option<String> { self.hash("sha512") }

	/// Returns an Iterator of URIs for the Version.
	pub fn uris(&self) -> impl Iterator<Item = String> + 'a {
		self.version_files().filter_map(|v| {
			let pkg_file = v.package_file();
			if !pkg_file.is_downloadable() {
				return None;
			}
			Some(pkg_file.index_file().archive_uri(&v.lookup().filename()))
		})
	}

	/// Set this version as the candidate.
	pub fn set_candidate(&self) { self.cache.depcache().set_candidate_version(self); }

	/// The priority of the Version as shown in `apt policy`.
	pub fn priority(&self) -> i32 { self.cache.priority(self) }
}

// Implementations for comparing versions.
impl<'a> PartialEq for Version<'a> {
	fn eq(&self, other: &Self) -> bool {
		matches!(
			cmp_versions(self.version(), other.version()),
			Ordering::Equal
		)
	}
}

impl<'a> Ord for Version<'a> {
	fn cmp(&self, other: &Self) -> Ordering { cmp_versions(self.version(), other.version()) }
}

impl<'a> PartialOrd for Version<'a> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl<'a> fmt::Display for Version<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.version())?;
		Ok(())
	}
}

impl<'a> fmt::Debug for Version<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let parent = self.parent();
		f.debug_struct("Version")
			.field("pkg", &parent.name())
			.field("arch", &self.arch())
			.field("version", &self.version())
			.field(
				"is_candidate",
				&parent.candidate().is_some_and(|cand| self == &cand),
			)
			.field("is_installed", &self.is_installed())
			.finish_non_exhaustive()
	}
}

#[cxx::bridge]
pub(crate) mod raw {
	impl CxxVector<VerIterator> {}
	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/package.h");

		type VerIterator;

		type PkgIterator = crate::iterators::PkgIterator;
		type PrvIterator = crate::iterators::PrvIterator;
		type DepIterator = crate::iterators::DepIterator;
		type DescIterator = crate::iterators::DescIterator;
		type VerFileIterator = crate::iterators::VerFileIterator;

		/// The version string of the version. "1.4.10".
		pub fn version(self: &VerIterator) -> &str;

		/// The Arch of the version. "amd64".
		pub fn arch(self: &VerIterator) -> &str;

		/// Return the version's parent PkgIterator.
		///
		/// # Safety
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn parent_pkg(self: &VerIterator) -> UniquePtr<PkgIterator>;

		/// The section of the version as shown in `apt show`.
		pub fn section(self: &VerIterator) -> Result<&str>;

		/// The priority string as shown in `apt show`.
		pub fn priority_str(self: &VerIterator) -> Result<&str>;

		/// The size of the .deb file.
		pub fn size(self: &VerIterator) -> u64;

		/// The uncompressed size of the .deb file.
		pub fn installed_size(self: &VerIterator) -> u64;

		// TODO: Possibly return an enum
		pub fn multi_arch(self: &VerIterator) -> u8;

		/// String representing MultiArch flag
		/// same, foreign, allowed, none
		pub fn multi_arch_type(self: &VerIterator) -> &str;

		/// True if the version is able to be downloaded.
		#[cxx_name = "Downloadable"]
		pub fn is_downloadable(self: &VerIterator) -> bool;

		/// True if the version is currently installed
		pub fn is_installed(self: &VerIterator) -> bool;

		/// Always contains the name, even if it is the same as the binary name
		pub fn source_name(self: &VerIterator) -> &str;

		// Always contains the version string,
		// even if it is the same as the binary version.
		pub fn source_version(self: &VerIterator) -> &str;

		/// Return Providers Iterator
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn provides(self: &VerIterator) -> UniquePtr<PrvIterator>;

		/// Return Dependency Iterator
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn depends(self: &VerIterator) -> UniquePtr<DepIterator>;

		/// Return the version files.
		/// You go through here to get the package files.
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn version_files(self: &VerIterator) -> UniquePtr<VerFileIterator>;

		/// This is for backend records lookups.
		///
		/// # Safety
		///
		/// If the inner pointer is null segfaults can occur.
		///
		/// Using [`crate::raw::IntoRawIter::make_safe`] to convert to an Option
		/// is recommended.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn translated_desc(self: &VerIterator) -> UniquePtr<DescIterator>;

		#[cxx_name = "Index"]
		pub fn index(self: &VerIterator) -> u64;
		/// Clone the pointer.
		///
		/// # Safety
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn unique(self: &VerIterator) -> UniquePtr<VerIterator>;
		pub fn raw_next(self: Pin<&mut VerIterator>);
		pub fn end(self: &VerIterator) -> bool;
	}
}
