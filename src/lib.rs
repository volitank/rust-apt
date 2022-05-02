//! rust-apt provides bindings to `libapt-pkg`.
//! The goal is to eventually have all of the functionality that `python-apt` has.
//!
//! The source repository is https://gitlab.com/volian/rust-apt
//! For more information please see the readme in the source code.

pub mod cache;
pub mod raw;

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn pretty_print_all() {
//         let mut cache = Cache::get_singleton();
//         let read_all_and_count = cache.iter().map(simple::BinaryPackageVersions::new).count();
//         assert!(read_all_and_count > 2);
//         assert_eq!(read_all_and_count, cache.iter().count());
//     }

//     #[test]
//     fn find_a_package() {
//         let mut cache = Cache::get_singleton();

//         if let Some(view) = cache.find_by_name("apt").next() {
//             assert_eq!("apt", view.name());
//         } else {
//             panic!("not found!");
//         }

//         assert!(cache
//             .find_by_name("this-package-doesnt-exist-and-if-someone-makes-it-ill-be-really-angry")
//             .next()
//             .is_none());
//     }

//     #[test]
//     fn compare_versions() {
//         use std::cmp::Ordering;
//         let cache = Cache::get_singleton();
//         assert_eq!(Ordering::Less, cache.compare_versions("3.0", "3.1"));
//         assert_eq!(Ordering::Greater, cache.compare_versions("3.1", "3.0"));
//         assert_eq!(Ordering::Equal, cache.compare_versions("3.0", "3.0"));
//     }

//     #[test]
//     fn reload() {
//         let mut cache = Cache::get_singleton();
//         cache.reload();
//         cache.reload();
//         cache.reload();
//         cache.reload();
//     }
// }
