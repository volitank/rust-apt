/// Initialize the apt configuration.
void init_config();

/// Initialize the apt system.
void init_system();

/// Returns a string dump of configuration options separated by `\n`
rust::string config_dump();
/// Find a key and return it's value as a string.
rust::string config_find(rust::string key, rust::string default_value);
/// Find a file and return it's value as a string.
rust::string config_find_file(rust::string key, rust::string default_value);
/// Find a directory and return it's value as a string.
rust::string config_find_dir(rust::string key, rust::string default_value);
/// Same as find, but for boolean values.
bool config_find_bool(rust::string key, bool default_value);
/// Same as find, but for i32 values.
int config_find_int(rust::string key, int default_value);
/// Return a vector for an Apt configuration list.
rust::vec<rust::string> config_find_vector(rust::string key);

/// Set the given key to the specified value.
void config_set(rust::string key, rust::string value);
/// Simply check if a key exists.
bool config_exists(rust::string key);
/// Clears all values from a key.
void config_clear(rust::string key);
/// Clears all configuratations.
void config_clear_all();
/// Clear a single value from a list.
/// Used for removing one item in an apt configuruation list
void config_clear_value(rust::string key, rust::string value);
