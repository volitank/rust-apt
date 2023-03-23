use std::io::prelude::*;
use std::{env, fs};
use std::fs::File;

fn main() {
	// Set up 'defines.h' for our features.
	let mut defines = File::open("apt-pkg-c/defines.h").unwrap();
	let mut defines_string = String::new();
	defines.read_to_string(&mut defines_string).unwrap();

	if env::var("CARGO_FEATURE_WORKER_SIZES").is_ok() {
		defines_string =
			defines_string.replace("RUST_APT_WORKER_SIZES 0", "RUST_APT_WORKER_SIZES 1");
	} else {
		defines_string =
			defines_string.replace("RUST_APT_WORKER_SIZES 1", "RUST_APT_WORKER_SIZES 0");
	}

	fs::write("apt-pkg-c/defines.h", defines_string).unwrap();

	let source_files = vec![
		"src/raw/package.rs",
		"src/raw/cache.rs",
		"src/raw/progress.rs",
		"src/raw/config.rs",
		"src/raw/util.rs",
		"src/raw/records.rs",
		"src/raw/depcache.rs",
		"src/raw/pkgmanager.rs",
	];

	cxx_build::bridges(source_files)
		.file("apt-pkg-c/progress.cc")
		.flag_if_supported("-std=c++14")
		.compile("rust-apt");

	println!("cargo:rustc-link-lib=apt-pkg");
	println!("cargo:rerun-if-changed=src/raw/cache.rs");
	println!("cargo:rerun-if-changed=src/raw/progress.rs");
	println!("cargo:rerun-if-changed=src/raw/config.rs");
	println!("cargo:rerun-if-changed=src/raw/util.rs");
	println!("cargo:rerun-if-changed=src/raw/records.rs");
	println!("cargo:rerun-if-changed=src/raw/depcache.rs");
	println!("cargo:rerun-if-changed=src/raw/package.rs");
	println!("cargo:rerun-if-changed=src/raw/pkgmanager.rs");

	println!("cargo:rerun-if-changed=apt-pkg-c/progress.cc");

	println!("cargo:rerun-if-changed=apt-pkg-c/cache.h");
	println!("cargo:rerun-if-changed=apt-pkg-c/progress.h");
	println!("cargo:rerun-if-changed=apt-pkg-c/configuration.h");
	println!("cargo:rerun-if-changed=apt-pkg-c/util.h");
	println!("cargo:rerun-if-changed=apt-pkg-c/records.h");
	println!("cargo:rerun-if-changed=apt-pkg-c/depcache.h");
	println!("cargo:rerun-if-changed=apt-pkg-c/package.h");
	println!("cargo:rerun-if-changed=apt-pkg-c/pkgmanager.h");
	println!("cargo:rerun-if-changed=apt-pkg-c/defines.h");
}
