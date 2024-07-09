//! Contains config related structs and functions.

use cxx::UniquePtr;

/// Struct for Apt Configuration
///
/// All apt configuration methods do not require this struct.
/// You can call the bindings directly from raw::apt if you would like.
#[derive(Debug)]
pub struct Config {}

// TODO: ConfigTree can (signal: 11, SIGSEGV: invalid memory reference)
// if you get a ConfigTree object and then clear the config with clear_all
// Make clear_all consume the Config struct and make ConfigTree have a lifetime
// to it?

impl Default for Config {
	/// Create a new config object and safely init the config system.
	///
	/// If you initialize the struct without `new()` or `default()`
	/// You will need to manually initialize the config system.
	fn default() -> Self { Self::new() }
}
// TODO: I think we should not accept &str if we just call to_string() anyway
impl Config {
	/// Create a new config object and safely init the config system.
	///
	/// If you initialize the struct without `new()` or `default()`
	/// You will need to manually initialize the config system.
	pub fn new() -> Self {
		init_config_system();
		Self {}
	}

	/// Clears all configuratations, re-initialize, and returns the config
	/// object.
	pub fn new_clear() -> Self {
		raw::clear_all();
		Self::new()
	}

	/// Resets the configurations.
	///
	/// If you'd like to clear everything and NOT reinit
	/// you can call `self.clear_all` or `raw::clear_all` directly
	pub fn reset(&self) {
		self.clear_all();
		init_config_system();
	}

	/// Clears all values from a key.
	///
	/// If the value is a list, the entire list is cleared.
	/// If you need to clear 1 value from a list see `self.clear_value`
	pub fn clear(&self, key: &str) { raw::clear(key.to_string()); }

	/// Clear a single value from a list.
	/// Used for removing one item in an apt configuruation list
	pub fn clear_value(&self, key: &str, value: &str) {
		raw::clear_value(key.to_string(), value.to_string());
	}

	/// Clears all configuratations.
	///
	/// This will leave you with an empty configuration object
	/// and most things probably won't work right.
	pub fn clear_all(&self) { raw::clear_all(); }

	/// Returns a string dump of configuration options separated by `\n`
	pub fn dump(&self) -> String { raw::dump() }

	/// Find a key and return it's value as a string.
	///
	/// default is what will be returned if nothing is found.
	pub fn find(&self, key: &str, default: &str) -> String {
		raw::find(key.to_string(), default.to_string())
	}

	/// Exactly like find but takes no default and returns an option instead.
	pub fn get(&self, key: &str) -> Option<String> {
		let value = raw::find(key.to_string(), "".to_string());
		if value.is_empty() {
			return None;
		}
		Some(value)
	}

	/// Find a file and return it's value as a string.
	///
	/// default is what will be returned if nothing is found.
	///
	/// `key = "Dir::Cache::pkgcache"` should return
	/// `/var/cache/apt/pkgcache.bin`
	///
	/// There is not much difference in `self.dir` and `self.file`
	///
	/// `dir` will return with a trailing `/` where `file` will not.
	pub fn file(&self, key: &str, default: &str) -> String {
		raw::find_file(key.to_string(), default.to_string())
	}

	/// Find a directory and return it's value as a string.
	///
	/// default is what will be returned if nothing is found.
	///
	/// `key = "Dir::Etc::sourceparts"` should return `/etc/apt/sources.list.d/`
	///
	/// There is not much difference in `self.dir` and `self.file`
	///
	/// `dir` will return with a trailing `/` where `file` will not.
	pub fn dir(&self, key: &str, default: &str) -> String {
		raw::find_dir(key.to_string(), default.to_string())
	}

	/// Same as find, but for boolean values.
	pub fn bool(&self, key: &str, default: bool) -> bool {
		raw::find_bool(key.to_string(), default)
	}

	/// Same as find, but for i32 values.
	pub fn int(&self, key: &str, default: i32) -> i32 { raw::find_int(key.to_string(), default) }

	/// Return a vector for an Apt configuration list.
	///
	/// An example of a common key that contains a list `raw::NeverAutoRemove`.
	pub fn find_vector(&self, key: &str) -> Vec<String> { raw::find_vector(key.to_string()) }

	/// Return a vector of supported architectures on this system.
	/// The main architecture is the first in the list.
	pub fn get_architectures(&self) -> Vec<String> { raw::get_architectures() }

	/// Simply check if a key exists.
	pub fn contains(&self, key: &str) -> bool { raw::exists(key.to_string()) }

	/// Set the given key to the specified value.
	pub fn set(&self, key: &str, value: &str) { raw::set(key.to_string(), value.to_string()) }

	pub fn tree(&self, key: &str) -> ConfigTree {
		ConfigTree::new(unsafe { raw::tree(key.to_string()) })
	}

	pub fn root_tree(&self) -> ConfigTree { ConfigTree::new(unsafe { raw::root_tree() }) }

	/// Add strings from a vector into an apt configuration list.
	///
	/// If the configuration key is not a list,
	/// you will receive a vector with one item.
	///
	/// Example:
	/// ```
	/// use rust_apt::config::Config;
	/// let config = Config::new();
	///
	/// let apt_list = vec!["This", "is", "my", "apt", "list"];
	/// // Using "AptList" here will not work and will panic.
	/// config.set_vector("AptList", &apt_list);
	/// ```
	pub fn set_vector(&self, key: &str, values: &Vec<&str>) {
		let mut vec_key = String::from(key);
		if !vec_key.ends_with("::") {
			vec_key.push_str("::");
		}

		for value in values {
			raw::set(vec_key.to_string(), value.to_string());
		}
	}
}

pub struct ConfigTree {
	pub ptr: UniquePtr<raw::ConfigTree>,
}

impl ConfigTree {
	pub fn new(ptr: UniquePtr<raw::ConfigTree>) -> Self { ConfigTree { ptr } }

	pub fn tag(&self) -> Option<String> {
		let tag = self.ptr.tag();
		if tag.is_empty() {
			return None;
		}
		Some(tag)
	}

	pub fn value(&self) -> Option<String> {
		let value = self.ptr.value();
		if value.is_empty() {
			return None;
		}
		Some(value)
	}

	pub fn child(&self) -> Option<ConfigTree> {
		let child = unsafe { self.ptr.child() };
		if child.end() { None } else { Some(ConfigTree::new(child)) }
	}

	pub fn sibling(&self) -> Option<ConfigTree> {
		let child = unsafe { self.ptr.raw_next() };
		if child.end() { None } else { Some(ConfigTree::new(child)) }
	}

	pub fn parent(&self) -> Option<ConfigTree> {
		let parent = unsafe { self.ptr.parent() };
		if parent.end() { None } else { Some(ConfigTree::new(parent)) }
	}

	pub fn iter(&self) -> IterConfigTree {
		IterConfigTree(unsafe { ConfigTree::new(self.ptr.unique()) })
	}
}

impl IntoIterator for ConfigTree {
	type IntoIter = IterConfigTree;
	type Item = ConfigTree;

	fn into_iter(self) -> Self::IntoIter { IterConfigTree(self) }
}

pub struct IterConfigTree(ConfigTree);

impl Iterator for IterConfigTree {
	type Item = ConfigTree;

	fn next(&mut self) -> Option<Self::Item> {
		if self.0.ptr.end() {
			None
		} else {
			let ret = unsafe { self.0.ptr.unique() };
			let next = unsafe { self.0.ptr.raw_next() };
			self.0.ptr = next;
			Some(ConfigTree::new(ret))
		}
	}
}

/// Safely Init Apt Configuration and System.
///
/// If the configuration has already been initialized, don't reinit.
///
/// This could cause some things to get reset.
pub fn init_config_system() {
	if !raw::exists("APT::Architecture".to_string()) {
		raw::init_config();
	}
	raw::init_system();
}

#[cxx::bridge]
pub(crate) mod raw {
	unsafe extern "C++" {
		include!("rust-apt/apt-pkg-c/configuration.h");

		type ConfigTree;

		/// init the system. This must occur before creating the cache.
		pub fn init_system();

		/// init the config. This must occur before creating the cache.
		pub fn init_config();

		/// Returns a string dump of configuration options separated by `\n`
		pub fn dump() -> String;

		/// Find a key and return it's value as a string.
		pub fn find(key: String, default_value: String) -> String;

		/// Find a file and return it's value as a string.
		pub fn find_file(key: String, default_value: String) -> String;

		/// Find a directory and return it's value as a string.
		pub fn find_dir(key: String, default_value: String) -> String;

		/// Same as find, but for boolean values.
		pub fn find_bool(key: String, default_value: bool) -> bool;

		/// Same as find, but for i32 values.
		pub fn find_int(key: String, default_value: i32) -> i32;

		/// Return a vector for an Apt configuration list.
		pub fn find_vector(key: String) -> Vec<String>;

		/// Return a vector of supported architectures on this system.
		/// The main architecture is the first in the list.
		pub fn get_architectures() -> Vec<String>;

		/// Set the given key to the specified value.
		pub fn set(key: String, value: String);

		/// Simply check if a key exists.
		pub fn exists(key: String) -> bool;

		/// Clears all values from a key.
		///
		/// If the value is a list, the entire list is cleared.
		/// If you need to clear 1 value from a list see `clear_value`
		pub fn clear(key: String);

		/// Clears all configuratations.
		pub fn clear_all();

		/// Clear a single value from a list.
		/// Used for removing one item in an apt configuruation list
		pub fn clear_value(key: String, value: String);

		unsafe fn tree(key: String) -> UniquePtr<ConfigTree>;
		unsafe fn root_tree() -> UniquePtr<ConfigTree>;

		pub fn end(self: &ConfigTree) -> bool;
		unsafe fn raw_next(self: &ConfigTree) -> UniquePtr<ConfigTree>;
		unsafe fn unique(self: &ConfigTree) -> UniquePtr<ConfigTree>;

		unsafe fn parent(self: &ConfigTree) -> UniquePtr<ConfigTree>;
		unsafe fn child(self: &ConfigTree) -> UniquePtr<ConfigTree>;
		pub fn tag(self: &ConfigTree) -> String;
		pub fn value(self: &ConfigTree) -> String;
	}
}
