#[macro_export]
/// Macro to create the cache, optionally including local valid files.
/// This includes the following:
/// - `*.deb` or `*.ddeb` files
/// - `Packages` and `Sources` files from apt repositories. These files can be
///   compressed.
/// - `*.dsc` or `*.changes` files
/// - A valid directory containing the file `./debian/control`
///
/// Here is an example of the two ways you can use this.
///
/// ```
/// use rust_apt::new_cache;
///
/// let cache = new_cache!().unwrap();
///
/// println!("{}", cache.get("apt").unwrap().name());
///
/// // Any file that can be added to the cache
/// let local_files = vec![
///     "tests/files/cache/apt.deb",
///     "tests/files/cache/Packages",
/// ];
///
/// let cache = new_cache!(&local_files).unwrap();
/// println!("{}", cache.get("apt").unwrap().get_version("5000:1.0.0").unwrap().version());
/// ```
///
/// Returns `Result<rust_apt::cache::Cache, AptErrors>`
macro_rules! new_cache {
	() => {{
		let files: Vec<String> = Vec::new();
		$crate::cache::Cache::new(&files)
	}};
	($slice:expr) => {{ $crate::cache::Cache::new($slice) }};
}
