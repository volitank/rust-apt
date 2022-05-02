# RUST-APT

rust-apt provides bindings to `libapt-pkg`.

Currently there isn't much functionality, only basic package querying.

The goal is to eventually have all of the functionality that `python-apt` has.

This is a fork of https://github.com/FauxFaux/apt-pkg-native-rs, which was originally designed
to function more like `libapt-pkg` itself. `rust-apt` will is designed to be more intuitive

A big thanks is in order for FauxFaux. His original crate is a huge contribution to this project.
It likely would not have gotten done with out him.

*This Crate is Under Active Development*
This API is far from what could be considered stable.
If you plan on using it in a real project make sure to pin the exact version.
Breaking changes will be frequent and potentially unnannouned as the API comes together.

Additionally, if you do anything 'wrong', `libapt-pkg` will just segfault.

# Documentation and Examples

This API doesn't have much documentation, but it's also not very complicated at the moment.
Here is a simple example of how you might use it.

### Getting a sorted package BTreeMap
`cache.sorted()` takes a struct that sorts the packages.
You will recieve a `BTreeMap<PkgName, Package>` sorted by package name.

These are sorted before the package objects are created, aking this extremely fast.
Currently only upgradable and virtual are supported.

`virtual_pkgs = false` will make sure not to include them.
`upgradable = true` will *ONLY* include packages that are upgradable.

```rust
use rust_apt::cache::{Cache, PackageSort};

let cache = Cache::new();

let sort = PackageSort{upgradable: true, virtual_pkgs: false};

for pkg in cache.sorted(sort).values() {
	println!("This Package is Upgradable! {}", pkg.name);
	if let Some(candidate) = pkg.candidate() {
		println!("{}", candidate);
	}
}
```

### Getting all packages
The `cache.packages()` method is an iterator of all packages, unordered and includes virtual.

Currently this is the slowest method of getting a package list.
It is planned to add a pre sorter to this much like the `cache.sorted()` but not sorting by package name.
This would then become the fastest method of getting a package list.

Here is how you could do something similar to the above example.

```rust
use rust_apt::cache::Cache;

let cache = Cache::new();

for pkg in cache.packages() {
  if pkg.is_upgradable() && pkg.has_versions {
    println!("{}", pkg.name)
  }
}
```

# License Note

This crate is licensed under the GPLv3 or later.

The original project was under the MIT License.
This license has been included in the source code to comply.

Basically all that remains from the original project are some of the C++ bindings. If
Your intentions are to use any of the code and maintain an MIT license you need to make
sure that you pull directly from the original project. Any code taken from here will need
to comply with the GPLv3 or later.

# Building

`libapt-pkg-dev` must be installed.

# Thread safety

It is not advised to use this crate in multiple threads. You're free to try it
but Development will not be focused on making this crate thread safe.
