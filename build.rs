fn main() {
	cxx_build::bridge("src/raw.rs")
		.file("apt-pkg-c/apt-pkg.cc")
		.file("apt-pkg-c/progress.cc")
		.file("apt-pkg-c/configuration.cc")
		.flag_if_supported("-std=c++14")
		.compile("rust-apt");

	println!("cargo:rustc-link-lib=apt-pkg");
	println!("cargo:rerun-if-changed=src/raw.rs");
	println!("cargo:rerun-if-changed=apt-pkg-c/apt-pkg.cc");
	println!("cargo:rerun-if-changed=apt-pkg-c/apt-pkg.h");
	println!("cargo:rerun-if-changed=apt-pkg-c/progress.cc");
	println!("cargo:rerun-if-changed=apt-pkg-c/progress.h");
	println!("cargo:rerun-if-changed=apt-pkg-c/configuration.cc");
}
