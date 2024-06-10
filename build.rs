fn main() {
	let source_files = vec![
		"src/cache.rs",
		"src/progress.rs",
		"src/config.rs",
		"src/util.rs",
		"src/records.rs",
		"src/depcache.rs",
		"src/pkgmanager.rs",
		"src/error.rs",
		"src/acquire.rs",
		"src/iterators/package.rs",
		"src/iterators/version.rs",
		"src/iterators/dependency.rs",
		"src/iterators/provider.rs",
		"src/iterators/files.rs",
	];

	let mut cc_files = vec!["apt-pkg-c/error.cc"];

	cxx_build::bridges(&source_files)
		.files(&cc_files)
		.flag_if_supported("-std=c++14")
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
		"apt-pkg-c/acquire.h",
	]);

	for file in cc_files {
		println!("cargo:rerun-if-changed={file}")
	}
}
