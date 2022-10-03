fn main() {
	let source_files = vec![
		"src/cache.rs",
		"src/progress.rs",
		"src/config.rs",
		"src/util.rs",
		"src/records.rs",
		"src/resolver.rs",
		"src/depcache.rs",
		"src/package.rs",
		"src/pkgmanager.rs",
	];

	cxx_build::bridges(source_files)
		.file("apt-pkg-c/cache.cc")
		.file("apt-pkg-c/progress.cc")
		.file("apt-pkg-c/configuration.cc")
		.file("apt-pkg-c/util.cc")
		.file("apt-pkg-c/records.cc")
		.file("apt-pkg-c/depcache.cc")
		.file("apt-pkg-c/package.cc")
		.file("apt-pkg-c/pkgmanager.cc")
		.file("apt-pkg-c/resolver.cc")
		.flag_if_supported("-std=c++14")
		.compile("rust-apt");

	println!("cargo:rustc-link-lib=apt-pkg");
	println!("cargo:rerun-if-changed=src/cache.rs");
	println!("cargo:rerun-if-changed=src/progress.rs");
	println!("cargo:rerun-if-changed=src/config.rs");
	println!("cargo:rerun-if-changed=src/util.rs");
	println!("cargo:rerun-if-changed=src/records.rs");
	println!("cargo:rerun-if-changed=src/depcache.rs");
	println!("cargo:rerun-if-changed=src/package.rs");
	println!("cargo:rerun-if-changed=src/pkgmanager.rs");
	println!("cargo:rerun-if-changed=src/resolver.rs");

	println!("cargo:rerun-if-changed=apt-pkg-c/cache.cc");
	println!("cargo:rerun-if-changed=apt-pkg-c/cache.h");

	println!("cargo:rerun-if-changed=apt-pkg-c/progress.cc");
	println!("cargo:rerun-if-changed=apt-pkg-c/progress.h");

	println!("cargo:rerun-if-changed=apt-pkg-c/configuration.cc");
	println!("cargo:rerun-if-changed=apt-pkg-c/configuration.h");

	println!("cargo:rerun-if-changed=apt-pkg-c/util.cc");
	println!("cargo:rerun-if-changed=apt-pkg-c/util.h");

	println!("cargo:rerun-if-changed=apt-pkg-c/records.cc");
	println!("cargo:rerun-if-changed=apt-pkg-c/records.h");

	println!("cargo:rerun-if-changed=apt-pkg-c/depcache.cc");
	println!("cargo:rerun-if-changed=apt-pkg-c/depcache.h");

	println!("cargo:rerun-if-changed=apt-pkg-c/package.cc");
	println!("cargo:rerun-if-changed=apt-pkg-c/package.h");

	println!("cargo:rerun-if-changed=apt-pkg-c/pkgmanager.cc");
	println!("cargo:rerun-if-changed=apt-pkg-c/pkgmanager.h");

	println!("cargo:rerun-if-changed=apt-pkg-c/resolver.cc");
	println!("cargo:rerun-if-changed=apt-pkg-c/resolver.h");
}
