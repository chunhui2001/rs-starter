use std::path::Path;

pub const ROOT_DIR: &'static str = env!("CARGO_MANIFEST_DIR");

pub fn temp_dir() -> std::string::String {
	std::env::temp_dir().as_path().display().to_string()
}

pub fn file_exists(file_path: &str) -> bool {
	if file_path.is_empty() {
		return false;
	}
	Path::new(file_path).exists()
}
