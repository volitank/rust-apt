#pragma once
#include <apt-pkg/aptconfiguration.h>
#include <apt-pkg/configuration.h>
#include <apt-pkg/init.h>
#include <apt-pkg/pkgsystem.h>
#include <sstream>
#include "rust/cxx.h"

#include "types.h"

/// The configuration pointer is global.
/// We do not need to make a new unique one.

/// Initialize the apt configuration.
void init_config() { pkgInitConfig(*_config); }
/// Initialize the apt system.

void init_system() { pkgInitSystem(*_config, _system); }

/// Returns a String dump of configuration options separated by `\n`
String dump() {
	std::stringstream String_stream;
	_config->Dump(String_stream);
	return String_stream.str();
}

/// Find a key and return it's value as a String.
String find(String key, String default_value) {
	return _config->Find(key.c_str(), default_value.c_str());
}

/// Find a file and return it's value as a String.
String find_file(String key, String default_value) {
	return _config->FindFile(key.c_str(), default_value.c_str());
}

/// Find a directory and return it's value as a String.
String find_dir(String key, String default_value) {
	return _config->FindDir(key.c_str(), default_value.c_str());
}

/// Same as find, but for boolean values.
bool find_bool(String key, bool default_value) {
	return _config->FindB(key.c_str(), default_value);
}

/// Same as find, but for i32 values.
int find_int(String key, i32 default_value) { return _config->FindI(key.c_str(), default_value); }

/// Return a vector for an Apt configuration list.
Vec<String> find_vector(String key) {
	std::vector<std::string> vector = _config->FindVector(key.c_str());
	Vec<String> rust_vector;

	for (const std::string& str : vector) {
		rust_vector.push_back(str);
	}

	return rust_vector;
}

/// Return a vector of supported architectures on this system.
/// The main architecture is the first in the list.
Vec<String> get_architectures() {
	Vec<String> rust_vector;

	for (const std::string& str : APT::Configuration::getArchitectures()) {
		rust_vector.push_back(str);
	}

	return rust_vector;
}

/// Set the given key to the specified value.
void set(String key, String value) { _config->Set(key.c_str(), value.c_str()); }

/// Simply check if a key exists.
bool exists(String key) { return _config->Exists(key.c_str()); }

/// Clears all values from a key.
///
/// If the value is a list, the entire list is cleared.
/// If you need to clear 1 value from a list see `clear_value`
void clear(String key) { _config->Clear(key.c_str()); }

/// Clear all configurations.
void clear_all() { _config->Clear(); }

/// Clear a single value from a list.
void clear_value(String key, String value) { _config->Clear(key.c_str(), value.c_str()); }
