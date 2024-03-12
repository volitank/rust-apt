/// This module contains the bindings and structs shared with c++
#[cxx::bridge]
pub mod raw {
	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/configuration.h");

		/// init the system. This must occur before creating the cache.
		pub fn init_system();

		/// init the config. This must occur before creating the cache.
		pub fn init_config();

		/// Returns a string dump of configuration options separated by `\n`
		pub fn config_dump() -> String;

		/// Find a key and return it's value as a string.
		pub fn config_find(key: String, default_value: String) -> String;

		/// Find a file and return it's value as a string.
		pub fn config_find_file(key: String, default_value: String) -> String;

		/// Find a directory and return it's value as a string.
		pub fn config_find_dir(key: String, default_value: String) -> String;

		/// Same as find, but for boolean values.
		pub fn config_find_bool(key: String, default_value: bool) -> bool;

		/// Same as find, but for i32 values.
		pub fn config_find_int(key: String, default_value: i32) -> i32;

		/// Return a vector for an Apt configuration list.
		pub fn config_find_vector(key: String) -> Vec<String>;

		/// Return a vector of supported architectures on this system.
		/// The main architecture is the first in the list.
		pub fn config_get_architectures() -> Vec<String>;

		/// Set the given key to the specified value.
		pub fn config_set(key: String, value: String);

		/// Simply check if a key exists.
		pub fn config_exists(key: String) -> bool;

		/// Clears all values from a key.
		///
		/// If the value is a list, the entire list is cleared.
		/// If you need to clear 1 value from a list see `config_clear_value`
		pub fn config_clear(key: String);

		/// Clears all configuratations.
		pub fn config_clear_all();

		/// Clear a single value from a list.
		/// Used for removing one item in an apt configuruation list
		pub fn config_clear_value(key: String, value: String);
	}
}
