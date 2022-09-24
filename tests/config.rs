mod config {
	use rust_apt::config::Config;

	#[test]
	fn clear() {
		// Test to make sure that the config populates properly.
		// Config will be empty if it hasn't been initialized.
		let config = Config::new_clear();
		config.clear_all();

		let empty_config = config.find("APT::Architecture", "");
		assert!(!config.contains("APT::Architecture"));
		assert!(empty_config.is_empty());

		// Reset the configuration which will clear and reinit.
		config.reset();

		// Now it should NOT be empty.
		let config_dump = config.find("APT::Architecture", "");
		assert!(config.contains("APT::Architecture"));
		assert!(!config_dump.is_empty());
		println!("{}", config.dump());
	}

	#[test]
	fn find_and_set() {
		let config = Config::new_clear();
		let key = "rust_apt::NotExist";

		// Find our key. It should not exist.
		assert_eq!(config.find(key, "None"), "None");

		// Set the key to something.
		config.set(key, "Exists!");

		// Find again and it should be there.
		assert_eq!(config.find(key, "None"), "Exists!");

		// Test other find functions on known defaults.
		assert!(!config.bool("APT::Install-Suggests", true));
		assert_eq!(config.int("APT::Install-Suggests", 20), 0);

		// Directory is different in CI. Just check for the name
		assert!(config
			.file("Dir::Cache::pkgcache", "")
			.split('/')
			.any(|x| x == "pkgcache.bin"));
		assert_eq!(
			config.dir("Dir::Etc::sourceparts", ""),
			"/etc/apt/sources.list.d/"
		);

		// Check if we can set a configuration list and retrieve it.
		// Make sure that the target vector is empty.
		assert!(config.find_vector("rust_apt::aptlist").is_empty());

		// Now fill our configuration vector and set it.
		let apt_list = vec!["this", "is", "my", "apt", "list"];
		config.set_vector("rust_apt::aptlist", &apt_list);

		// Retrieve a new vector from the configuration.
		let apt_vector = config.find_vector("rust_apt::aptlist");

		// If everything went smooth, our original vector should match the new one
		assert_eq!(apt_list, apt_vector);

		// Now test if we can remove a single value from the list.
		config.clear_value("rust_apt::aptlist", "my");

		// This will let us know if it worked!
		assert_eq!(
			config.find_vector("rust_apt::aptlist"),
			vec!["this", "is", "apt", "list"]
		);

		// Finally test and see if we can clear the entire list.
		config.clear("rust_apt::aptlist");
		assert!(config.find_vector("rust_apt::aptlist").is_empty());
	}
}
