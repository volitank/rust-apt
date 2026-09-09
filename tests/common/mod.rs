use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

pub fn cache_dir() -> &'static Path {
	static CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();

	CACHE_DIR.get_or_init(|| {
		let dir =
			Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cache-{}", std::process::id()));
		let status = Command::new(concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/tests/create-test-debs"
		))
		.arg(&dir)
		.status()
		.expect("failed to run tests/create-test-debs");

		assert!(status.success(), "fixture generation failed: {status}");
		dir
	})
}

pub fn cache_file(name: &str) -> String { cache_dir().join(name).to_string_lossy().into_owned() }
