//! Allows access to complete package description records directly from the
//! file.
use std::cell::{Ref, RefCell};

use cxx::UniquePtr;

// TODO: Probably just make this a real enum
// we an add a variant RecordField::String("Package".to_string())
// or something like that.

/// A module containing [`&str`] constants for known record fields
///
/// Pass through to the [`crate::Version::get_record`] method
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

pub struct PackageRecords {
	pub(crate) ptr: UniquePtr<raw::PkgRecords>,
	parser: RefCell<UniquePtr<raw::Parser>>,
	index: RefCell<u64>,
}

impl PackageRecords {
	pub fn new(ptr: UniquePtr<raw::PkgRecords>) -> PackageRecords {
		PackageRecords {
			ptr,
			parser: RefCell::new(UniquePtr::null()),
			index: RefCell::new(0),
		}
	}

	fn replace_index(&self, index: u64) -> bool {
		if self.index.borrow().eq(&index) {
			return false;
		}
		self.index.replace(index);
		true
	}

	fn parser(&self) -> Ref<'_, UniquePtr<raw::Parser>> {
		if self.parser.borrow().is_null() {
			panic!("You must call ver_lookup or desc_lookup first!")
		}
		self.parser.borrow()
	}

	pub fn ver_lookup(&self, file: &raw::VerFileIterator) -> &PackageRecords {
		if self.replace_index(file.index()) {
			unsafe { self.parser.replace(self.ptr.ver_lookup(file)) };
		}
		self
	}

	pub fn desc_lookup(&self, file: &raw::DescIterator) -> &PackageRecords {
		if self.replace_index(file.index()) {
			unsafe { self.parser.replace(self.ptr.desc_lookup(file)) };
		}
		self
	}

	pub fn short_desc(&self) -> Option<String> { self.parser().short_desc().ok() }

	pub fn long_desc(&self) -> Option<String> { self.parser().long_desc().ok() }

	pub fn filename(&self) -> String { self.parser().filename() }

	pub fn get_field(&self, field: String) -> Option<String> { self.parser().get_field(field).ok() }

	pub fn hash_find(&self, hash_type: String) -> Option<String> {
		self.parser().hash_find(hash_type).ok()
	}
}

type SourceParser<'a> = Ref<'a, UniquePtr<raw::SourceParser>>;

pub struct SourceRecords {
	ptr: UniquePtr<raw::SourceRecords>,
	parser: RefCell<UniquePtr<raw::SourceParser>>,
}

impl SourceRecords {
	pub fn new(ptr: UniquePtr<raw::SourceRecords>) -> SourceRecords {
		SourceRecords {
			ptr,
			parser: RefCell::new(UniquePtr::null()),
		}
	}

	/// Return all of the parsers to their starting position
	pub fn restart(&self) { self.ptr.restart() }

	/// Lookup the string name of the package.
	///
	/// # Example:
	/// ```
	/// use rust_apt::new_cache;
	///
	/// let cache = new_cache!().unwrap();
	/// let Ok(src_records) = cache.source_records() else {
	///     // Requires at least one `deb-src` entry in APT sources.
	///     return;
	/// };
	///
	/// while let Some(record) = src_records.lookup("libapt-pkg-dev".to_string(), false) {
	///     println!("{}", record.package());
	/// }
	/// ```
	pub fn lookup(&self, name: String, src_only: bool) -> Option<SourceParser<'_>> {
		unsafe {
			self.parser.replace(self.ptr.find(name, src_only));
		}

		if self.parser.borrow().end() {
			self.restart();
			return None;
		}
		Some(self.parser.borrow())
	}
}

#[cxx::bridge]
pub(crate) mod raw {
	impl UniquePtr<IndexFile> {}
	impl UniquePtr<SourceRecords> {}
	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/records.h");
		type PkgRecords;
		type Parser;
		type SourceRecords;
		type SourceParser;
		type IndexFile;
		type VerFileIterator = crate::iterators::VerFileIterator;
		type DescIterator = crate::iterators::DescIterator;

		/// Move the records into the correct spot for the Version File.
		///
		/// # Safety
		///
		/// The returned Parser can not out live the records struct.
		/// If you hold a Parser and lookup another file, the data that parser
		/// returns will change.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn ver_lookup(self: &PkgRecords, ver_file: &VerFileIterator) -> UniquePtr<Parser>;

		/// Move the records into the correct spot for the Description File.
		///
		/// # Safety
		///
		/// The returned Parser can not out live the records struct.
		/// If you hold a Parser and lookup another file, the data that parser
		/// returns will change.
		///
		/// The returned UniquePtr cannot outlive the cache.
		unsafe fn desc_lookup(self: &PkgRecords, desc_file: &DescIterator) -> UniquePtr<Parser>;

		pub fn filename(self: &Parser) -> String;
		pub fn long_desc(self: &Parser) -> Result<String>;
		pub fn short_desc(self: &Parser) -> Result<String>;

		pub fn get_field(self: &Parser, field: String) -> Result<String>;
		pub fn hash_find(self: &Parser, hash_type: String) -> Result<String>;

		pub fn archive_uri(self: &IndexFile, filename: &str) -> String;

		/// Return true if the IndexFile is trusted.
		pub fn is_trusted(self: &IndexFile) -> bool;

		pub fn restart(self: &SourceRecords);

		/// # Safety
		///
		/// The returned Parser can not out live the records struct.
		/// Make sure to check the `end()` to see if null.
		unsafe fn find(
			self: &SourceRecords,
			name: String,
			src_only: bool,
		) -> UniquePtr<SourceParser>;

		fn as_str(self: &SourceParser) -> String;
		fn package(self: &SourceParser) -> String;
		fn version(self: &SourceParser) -> String;
		fn maintainer(self: &SourceParser) -> String;
		fn section(self: &SourceParser) -> String;
		fn end(self: &SourceParser) -> bool;
	}
}
