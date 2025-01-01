use serde::Serialize;
use serde::ser::{SerializeSeq, SerializeStruct, Serializer};

use crate::records::RecordField;
use crate::{BaseDep, Dependency, PackageFile, Version};

const RECORDS: [&str; 13] = [
	RecordField::Package,
	RecordField::Version,
	RecordField::Architecture,
	RecordField::Priority,
	RecordField::Essential,
	RecordField::Section,
	RecordField::Source,
	RecordField::InstalledSize,
	RecordField::Size,
	RecordField::Maintainer,
	RecordField::OriginalMaintainer,
	RecordField::Homepage,
	RecordField::SHA256,
];

impl<'a> Serialize for Version<'a> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let depends = self.depends_map();
		let mut state = serializer.serialize_struct("Version", RECORDS.len() + depends.len())?;

		let vf = self.version_files().next().unwrap();
		let records = vf.lookup();
		for key in RECORDS {
			let Some(value) = records.get_field(key.to_string()) else {
				continue;
			};

			state.serialize_field(key, &value)?;
		}

		let pkg_files: Vec<PackageFile<'a>> = self.package_files().collect();
		state.serialize_field("package_files", &pkg_files)?;

		// Format Depends better
		for (kind, dep_vec) in self.depends_map() {
			state.serialize_field(kind.to_str(), &dep_vec)?;
		}

		state.end()
	}
}

impl Serialize for BaseDep<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut state = serializer.serialize_struct("Dependency", 3)?;

		state.serialize_field("name", &self.name())?;
		state.serialize_field("comp", &self.comp_type())?;
		state.serialize_field("version", &self.version())?;
		state.end()
	}
}

impl Serialize for PackageFile<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut state = serializer.serialize_struct("PackageFile", 5)?;
		state.serialize_field("filename", &self.filename())?;
		state.serialize_field("archive", &self.archive())?;
		state.serialize_field("origin", &self.origin())?;
		state.serialize_field("codename", &self.codename())?;
		state.serialize_field("component", &self.component())?;
		state.end()
	}
}

impl Serialize for Dependency<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut state = serializer.serialize_seq(Some(self.ptr.len()))?;
		for dep in &self.ptr {
			state.serialize_element(dep)?;
		}
		state.end()
	}
}
