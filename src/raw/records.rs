/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {
	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/package.h");
		include!("rust-apt/apt-pkg-c/records.h");
		type Records;
		type VersionFile = crate::raw::package::raw::VersionFile;
		type DescriptionFile = crate::raw::package::raw::DescriptionFile;
		type PackageFile = crate::raw::package::raw::PackageFile;

		pub fn ver_file_lookup(self: &Records, ver_file: &VersionFile);
		pub fn desc_file_lookup(self: &Records, desc_file: &DescriptionFile);

		pub fn long_desc(self: &Records) -> Result<String>;
		pub fn short_desc(self: &Records) -> Result<String>;

		pub fn get_field(self: &Records, field: String) -> Result<String>;
		pub fn hash_find(self: &Records, hash_type: String) -> Result<String>;

		pub fn ver_uri(self: &Records, pkg_file: &PackageFile) -> Result<String>;
	}
}
