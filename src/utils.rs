use rocket::fs::relative;
use std::path::{Path, PathBuf};

/// Converts a string into a path buffer.
///
/// Arguments:
///
/// * `path` - String
///
/// Usage: ```get_file_path(path);```
pub fn get_file_path(path: String) -> PathBuf {
  Path::new(relative!("static")).join(path.clone())
}

/// Converts a path buffer into a string.
///
/// Arguments:
///
/// * `path` - PathBuf
///
/// Usage: ```get_string_path(path);```
pub fn get_string_path(path: PathBuf) -> String {
  path.into_os_string().into_string().unwrap()
}
