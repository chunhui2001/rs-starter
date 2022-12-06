use std::path::Path;

pub const ROOT_DIR: &'static str = env!("CARGO_MANIFEST_DIR");

pub fn file_exists(file_path: &str) -> bool {
	if file_path.is_empty() {
		return false;
	}
	Path::new(file_path).exists()
}
