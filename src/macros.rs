#[macro_export]
/// Macro to create the cache, optionally including local valid files.
///
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
/// Returns [`Result<rust_apt::cache::Cache, rust_apt::error::AptErrors>`]
macro_rules! new_cache {
	() => {{
		let files: Vec<String> = Vec::new();
		$crate::cache::Cache::new(&files)
	}};
	($slice:expr) => {{ $crate::cache::Cache::new($slice) }};
}

/// Implements RawIter trait for raw apt iterators
macro_rules! raw_iter {
	($($ty:ty),*) => {$(
		paste!(
			#[doc = "Iterator Struct for [`" $ty "`]."]
			pub struct [<Iter $ty>](UniquePtr<$ty>);

			impl Iterator for [<Iter $ty>] {
				type Item = UniquePtr<$ty>;

				fn next(&mut self) -> Option<Self::Item> {
					if self.0.end() {
						None
					} else {
						let ptr = unsafe { self.0.unique() };
						self.0.pin_mut().raw_next();
						Some(ptr)
					}
				}
			}

			impl IntoRawIter for UniquePtr<$ty> {
				type Item = [<Iter $ty>];

				fn raw_iter(self) -> Self::Item { [<Iter $ty>](self) }

				fn make_safe(self) -> Option<Self> { if self.end() { None } else { Some(self) } }

				fn to_vec(self) -> Vec<Self> { self.raw_iter().collect() }
			}
		);
	)*};
}

/// Generates the boiler plate for wrapper structs
/// where we need to change a Result to an option.
macro_rules! cxx_convert_result {
	($wrapper:ident, $($(#[$meta:meta])* $method:ident ( $( $arg:ident : $arg_ty:ty ),* ) -> $ret:ty ),* $(,)? ) => {
		impl<'a> $wrapper<'a> {
			$(
				$(#[$meta])*
				pub fn $method(&self, $( $arg : $arg_ty ),* ) -> Option<$ret> {
					self.ptr.$method($( $arg ),*).ok()
				}
			)*
		}
	};
}

macro_rules! impl_partial_eq {
	($($wrapper:ident $(<$lt:lifetime>)?),* $(,)?) => {
		$(
			impl $(<$lt>)? PartialEq for $wrapper $(<$lt>)? {
				fn eq(&self, other: &Self) -> bool { self.index() == other.index() }
			}
		)*
	};
}

macro_rules! impl_hash_eq {
	($($wrapper:ident $(<$lt:lifetime>)?),* $(,)?) => {
		$(
			impl $(<$lt>)? std::hash::Hash for $wrapper $(<$lt>)? {
				fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.index().hash(state); }
			}

			impl $(<$lt>)? Eq for $wrapper $(<$lt>)? {}
		)*
	};
}

/// Implements deref for apt smart pointer structs.
macro_rules! impl_deref {
	($($wrapper:ident $(<$lt:lifetime>)? -> $target:ty),* $(,)?) => {
		$(
			impl $(<$lt>)? std::ops::Deref for $wrapper $(<$lt>)? {
				type Target = $target;

				#[inline]
				fn deref(&self) -> &Self::Target {
					&self.ptr
				}
			}
		)*
	};
}
