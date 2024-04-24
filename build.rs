fn main() {
	let source_files = vec![
		"src/raw/package.rs",
		"src/raw/cache.rs",
		"src/raw/progress.rs",
		"src/raw/config.rs",
		"src/raw/util.rs",
		"src/raw/records.rs",
		"src/raw/depcache.rs",
		"src/raw/pkgmanager.rs",
		"src/raw/error.rs",
	];

	let mut cc_files = vec!["apt-pkg-c/progress.cc", "apt-pkg-c/error.cc"];

	cxx_build::bridges(&source_files)
		.files(&cc_files)
		.flag_if_supported("-std=c++17")
		.compile("rust-apt");

	println!("cargo:rustc-link-lib=apt-pkg");
	for file in source_files {
		println!("cargo:rerun-if-changed={file}")
	}

	cc_files.extend_from_slice(&[
		"apt-pkg-c/cache.h",
		"apt-pkg-c/progress.h",
		"apt-pkg-c/configuration.h",
		"apt-pkg-c/util.h",
		"apt-pkg-c/records.h",
		"apt-pkg-c/depcache.h",
		"apt-pkg-c/package.h",
		"apt-pkg-c/pkgmanager.h",
		"apt-pkg-c/error.h",
		"apt-pkg-c/types.h",
	]);

	for file in cc_files {
		println!("cargo:rerun-if-changed={file}")
	}
}
