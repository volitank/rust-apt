# RUST-APT

`rust-apt` provides bindings to `libapt-pkg`.

Currently `rust-apt` has most functionality available such as basic querying of package information,
Installing and removing packages, updating the package lists and upgrading the system.

If you find something missing, please make an Issue to request the feature.

### *This Crate is Under Active Development*

This API is not considered stable. Breaking changes will happen as the API comes together.
As `rust-apt` doesn't have a Major, breaking change will be on the Minor. Never on the Patch.

`src/raw` contains the direct C++ bindings to `libapt-pkg` that are defined in `apt-pkg-c`

These are generally considered safe, but may cause segfaults if you do something wrong.
We offer no safety guarantees for using the `raw` bindings directly.

If you find a way to segfault without using the `raw` bindings directly, please report this as a bug.

# Documentation and Examples

For more instructions on how to use `rust-apt` see our [crates.io](https://crates.io/crates/rust-apt) page.

# License Note

This crate is licensed under the GPLv3 or later.

# Building

`libapt-pkg-dev` must be installed.

# Thread safety

It is not advised to use this crate in multiple threads.

You're free to try it but development will not be focused on making this crate thread safe.

# Development

Make sure `cargo` and `rustup` are installed before you run the following commands.

You will need the stable and nightly toolchain. Nightly is only used for `rustfmt`.

Install `just`, a command runner we use to simplify some tasks.

```console
cargo install just
```

Now that `cargo` and `just` are installed, You can setup your dev environment.

`setup-dev` will:

* Install the necessary dependencies with `apt`.

* Ensure the proper toolchains are installed with `rustup`.

* Create `compile_commands.json` with `bear` for better c++ linting

```console
just setup-dev
```

Before you commit, check formatting and basic code QA.

```console
just fmt
just check
```
