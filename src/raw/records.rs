/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {
	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/records.h");
		type PkgRecords;
		type VerFileIterator = crate::raw::package::raw::VerFileIterator;
		type DescFileIterator = crate::raw::package::raw::DescFileIterator;
		type IndexFile = crate::raw::package::raw::IndexFile;

		pub fn ver_file_lookup(self: &PkgRecords, ver_file: &VerFileIterator);
		pub fn desc_file_lookup(self: &PkgRecords, desc_file: &DescFileIterator);

		pub fn long_desc(self: &PkgRecords) -> Result<String>;
		pub fn short_desc(self: &PkgRecords) -> Result<String>;

		pub fn get_field(self: &PkgRecords, field: String) -> Result<String>;
		pub fn hash_find(self: &PkgRecords, hash_type: String) -> Result<String>;

		pub fn ver_uri(self: &PkgRecords, file: &UniquePtr<IndexFile>) -> Result<String>;
	}
}
