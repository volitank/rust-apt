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
		let mut state =
			serializer.serialize_struct("Version", RECORDS.len() + depends.len() + 1)?;

		let mut record_values: Vec<Option<String>> = vec![None; RECORDS.len()];
		for vf in self.version_files() {
			let records = vf.lookup();
			for (idx, key) in RECORDS.iter().enumerate() {
				if record_values[idx].is_some() {
					continue;
				}
				record_values[idx] = records.get_field((*key).to_string());
			}

			if record_values.iter().all(|value| value.is_some()) {
				break;
			}
		}

		for (key, value) in RECORDS.iter().zip(record_values.into_iter()) {
			let Some(value) = value else {
				continue;
			};
			state.serialize_field(key, &value)?;
		}

		let pkg_files: Vec<PackageFile<'a>> = self.package_files().collect();
		state.serialize_field("package_files", &pkg_files)?;

		// Format Depends better
		for (kind, dep_vec) in depends {
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
		let archive = self.archive();
		let mut state = serializer.serialize_struct("PackageFile", 11)?;
		state.serialize_field("priority", &self.priority())?;
		state.serialize_field("filename", &self.filename())?;
		state.serialize_field("archive", &archive)?;
		state.serialize_field("origin", &self.origin())?;
		state.serialize_field("codename", &self.codename())?;
		state.serialize_field("label", &self.label())?;
		state.serialize_field("component", &self.component())?;
		state.serialize_field("arch", &self.arch())?;
		state.serialize_field("site", &self.site())?;
		state.serialize_field("index_type", &self.index_type())?;
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
