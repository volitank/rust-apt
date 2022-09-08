//! Contains Records struct for getting extra information about a package.
//! Some of these functions are accessible on [`crate::package::Package`]
//! structs, please see if that suits your needs first. If not, you can also
//! access a [`Records`] struct on any [`crate::cache::Cache`] struct via
//! [`crate::cache::Cache::records`].
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use cxx::UniquePtr;

/// A module containing [`&str`] constants for known record fields
///
/// Pass through to the [`crate::package::Version::get_record`] method
/// or you can use a custom [`&str`] like the ones listed below.
///
/// Other Known Record Keys:
///
/// `Conffiles` `Status` `Python-Version` `Auto-Built-Package`
/// `Enhances` `Cnf-Extra-Commands` `Gstreamer-Elements`
/// `Gstreamer-Encoders` `Lua-Versions` `Original-Maintainer` `Protected`
/// `Gstreamer-Uri-Sources` `Vendor` `Build-Ids` `Efi-Vendor` `SHA512`
/// `Build-Essential` `Important` `X-Cargo-Built-Using`
/// `Cnf-Visible-Pkgname` `Gstreamer-Decoders` `SHA1` `Gstreamer-Uri-Sinks`
/// `Gstreamer-Version` `Ghc-Package` `Static-Built-Using`
/// `Postgresql-Catversion` `Python-Egg-Name` `Built-Using` `License`
/// `Cnf-Ignore-Commands` `Go-Import-Path` `Ruby-Versions`
#[allow(non_upper_case_globals, non_snake_case)]
pub mod RecordField {
	/// Name of the package `apt`
	pub const Package: &str = "Package";

	/// The name of the source package and the version if it exists
	/// `zsh (5.9-1)`
	// TODO: We need to write a parser to be able to handle this properly
	// The apt source that does this is in debrecords.cc
	pub const Source: &str = "Source";

	/// Version of the package `2.5.2`
	pub const Version: &str = "Version";

	/// The unpacked size in KiB? `4352`
	pub const InstalledSize: &str = "Installed-Size";

	/// The homepage of the software
	/// `https://gitlab.com/volian/rust-apt`
	pub const Homepage: &str = "Homepage";

	/// If the package is essential `yes`
	pub const Essential: &str = "Essential";

	/// The Maintainer of the package
	/// `APT Development Team <deity@lists.debian.org>`
	pub const Maintainer: &str = "Maintainer";

	/// The Original Maintainer of the package.
	/// Most common to see on Ubuntu packages repackaged from Debian
	/// `APT Development Team <deity@lists.debian.org>`
	pub const OriginalMaintainer: &str = "Original-Maintainer";

	/// The Architecture of the package `amd64`
	pub const Architecture: &str = "Architecture";

	/// Packages that this one replaces
	/// `apt-transport-https (<< 1.5~alpha4~), apt-utils (<< 1.3~exp2~)`
	pub const Replaces: &str = "Replaces";

	/// Packages that this one provides
	/// `apt-transport-https (= 2.5.2)`
	pub const Provides: &str = "Provides";

	/// Packages that must be installed and configured before this one
	/// `libc6 (>= 2.34), libtinfo6 (>= 6)`
	pub const PreDepends: &str = "Pre-Depends";

	/// Packages this one depends on
	/// `adduser, gpgv | gpgv2 | gpgv1, libapt-pkg6.0 (>= 2.5.2)`
	pub const Depends: &str = "Depends";

	/// Packages that are recommended to be installed with this one
	/// `ca-certificates`
	pub const Recommends: &str = "Recommends";

	/// Packages that are suggested to be installed with this one
	/// `apt-doc, aptitude | synaptic | wajig, dpkg-dev (>= 1.17.2)`
	pub const Suggests: &str = "Suggests";

	/// Packages that are broken by installing this.
	/// `apt-transport-https (<< 1.5~alpha4~), apt-utils (<< 1.3~exp2~)`
	pub const Breaks: &str = "Breaks";

	/// Packages that conflict with this one
	/// `bash-completion (<< 20060301-0)`
	pub const Conflicts: &str = "Conflicts";

	/// The raw description of the package
	/// `commandline package manager`
	pub const Description: &str = "Description";

	/// The MD5 sum of the description
	/// `9fb97a88cb7383934ef963352b53b4a7`
	pub const DescriptionMD5: &str = "Description-md5";

	/// Any tags associated with this package
	/// `admin::package-management, devel::lang:ruby, hardware::storage`
	pub const Tag: &str = "Tag";

	/// The type of multi arch for the package.
	/// Either `allowed`, `foreign`, or `same`
	pub const MultiArch: &str = "Multi-Arch";

	/// The section of the package `admin`
	pub const Section: &str = "Section";

	/// The Priority of the package `required`
	pub const Priority: &str = "Priority";

	/// The raw filename of the package
	/// `pool/main/a/apt/apt_2.5.2_amd64.deb`
	pub const Filename: &str = "Filename";

	/// The compressed size of the .deb in bytes `1500520`
	pub const Size: &str = "Size";

	/// The MD5 sum of the package `8797c5716952fba7779bd072e53acee5`
	pub const MD5sum: &str = "MD5sum";

	/// The SHA256 sum of the package
	/// `a6dd99a52ec937faa20e1617da36b8b27a2ed8bc9300bf7eb8404041ede52200`
	pub const SHA256: &str = "SHA256";
}

/// Internal Struct for managing package records.
#[derive(Debug)]
pub struct Records {
	pub(crate) ptr: raw::Records,
	pub(crate) cache: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>,
}

impl Records {
	pub fn new(cache: Rc<RefCell<UniquePtr<raw::PkgCacheFile>>>) -> Self {
		let record = raw::records_create(&cache.borrow());
		Records { ptr: record, cache }
	}

	pub fn lookup_desc(&mut self, desc: &UniquePtr<raw::DescIterator>) {
		// TODO: Do we actually need this? Is there a better way?
		// It seems like lookup_ver gets us the same information maybe
		// Currently this is only used for summary and description
		raw::desc_file_lookup(&mut self.ptr, desc);
	}

	pub fn lookup_ver(&mut self, ver_file: &raw::VersionFile) {
		raw::ver_file_lookup(&mut self.ptr, ver_file);
	}

	pub fn description(&self) -> Option<String> { raw::long_desc(&self.ptr).ok() }

	pub fn summary(&self) -> Option<String> { raw::short_desc(&self.ptr).ok() }

	/// Return the Source package version string.
	pub fn get_field(&self, field_name: String) -> Option<String> {
		raw::get_field(&self.ptr, field_name).ok()
	}

	pub fn uri(&self, pkg_file: &raw::VersionFile) -> String {
		raw::ver_uri(&self.ptr, &self.cache.borrow(), pkg_file)
	}

	pub fn hash_find(&self, hash_type: &str) -> Option<String> {
		if let Ok(hash) = raw::hash_find(&self.ptr, hash_type.to_string()) {
			return Some(hash);
		}
		None
	}
}

/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {
	/// A wrapper around the Apt pkgRecords class.
	struct Records {
		records: UniquePtr<PkgRecords>,
	}

	unsafe extern "C++" {
		type PkgRecords;

		type VersionFile = crate::cache::raw::VersionFile;
		type PackageFile = crate::cache::raw::PackageFile;
		type PkgCacheFile = crate::cache::raw::PkgCacheFile;
		type DescIterator = crate::cache::raw::DescIterator;

		include!("rust-apt/apt-pkg-c/cache.h");
		include!("rust-apt/apt-pkg-c/records.h");

		/// Package Record Management:

		/// Create the Package Records.
		pub fn records_create(cache: &UniquePtr<PkgCacheFile>) -> Records;

		/// Moves the Records into the correct place.
		pub fn ver_file_lookup(records: &mut Records, pkg_file: &VersionFile);

		/// Moves the Records into the correct place.
		pub fn desc_file_lookup(records: &mut Records, desc: &UniquePtr<DescIterator>);

		/// Return the URI for a version as determined by it's package file.
		/// A version could have multiple package files and multiple URIs.
		pub fn ver_uri(
			records: &Records,
			cache: &UniquePtr<PkgCacheFile>,
			ver_file: &VersionFile,
		) -> String;

		/// Return the translated long description of a Package.
		pub fn long_desc(records: &Records) -> Result<String>;

		/// Return the translated short description of a Package.
		pub fn short_desc(records: &Records) -> Result<String>;

		/// Return the Source package version string.
		pub fn get_field(records: &Records, field_name: String) -> Result<String>;

		/// Find the hash of a Version.
		// TODO: What kind of errors can be returned here?
		// Research and update higher level structs as well
		// TODO: Create custom errors when we have better information
		pub fn hash_find(records: &Records, hash_type: String) -> Result<String>;
	}
}

impl fmt::Debug for raw::Records {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Records: {{ To Be Implemented }}")?;
		Ok(())
	}
}
