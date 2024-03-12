#pragma once
#include <apt-pkg/aptconfiguration.h>
#include <apt-pkg/configuration.h>
#include <apt-pkg/init.h>
#include <apt-pkg/pkgsystem.h>
#include <sstream>
#include "rust/cxx.h"

/// The configuration pointer is global.
/// We do not need to make a new unique one.

/// Initialize the apt configuration.
void init_config() { pkgInitConfig(*_config); }
/// Initialize the apt system.

void init_system() { pkgInitSystem(*_config, _system); }

/// Returns a string dump of configuration options separated by `\n`
rust::string config_dump() {
	std::stringstream string_stream;
	_config->Dump(string_stream);
	return string_stream.str();
}

/// Find a key and return it's value as a string.
rust::string config_find(rust::string key, rust::string default_value) {
	return _config->Find(key.c_str(), default_value.c_str());
}

/// Find a file and return it's value as a string.
rust::string config_find_file(rust::string key, rust::string default_value) {
	return _config->FindFile(key.c_str(), default_value.c_str());
}

/// Find a directory and return it's value as a string.
rust::string config_find_dir(rust::string key, rust::string default_value) {
	return _config->FindDir(key.c_str(), default_value.c_str());
}

/// Same as find, but for boolean values.
bool config_find_bool(rust::string key, bool default_value) {
	return _config->FindB(key.c_str(), default_value);
}

/// Same as find, but for i32 values.
int config_find_int(rust::string key, int default_value) {
	return _config->FindI(key.c_str(), default_value);
}

/// Return a vector for an Apt configuration list.
rust::vec<rust::string> config_find_vector(rust::string key) {
	std::vector<std::string> config_vector = _config->FindVector(key.c_str());
	rust::vec<rust::string> rust_vector;

	for (const std::string& str : config_vector) {
		rust_vector.push_back(str);
	}

	return rust_vector;
}

/// Return a vector of supported architectures on this system.
/// The main architecture is the first in the list.
rust::vec<rust::string> config_get_architectures() {
	rust::vec<rust::string> rust_vector;

	for (const std::string& str : APT::Configuration::getArchitectures()) {
		rust_vector.push_back(str);
	}

	return rust_vector;
}

/// Set the given key to the specified value.
void config_set(rust::string key, rust::string value) { _config->Set(key.c_str(), value.c_str()); }

/// Simply check if a key exists.
bool config_exists(rust::string key) { return _config->Exists(key.c_str()); }

/// Clears all values from a key.
///
/// If the value is a list, the entire list is cleared.
/// If you need to clear 1 value from a list see `config_clear_value`
void config_clear(rust::string key) { _config->Clear(key.c_str()); }

/// Clear all configurations.
void config_clear_all() { _config->Clear(); }

/// Clear a single value from a list.
void config_clear_value(rust::string key, rust::string value) {
	_config->Clear(key.c_str(), value.c_str());
}
