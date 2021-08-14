use rocket::fs::relative;
use std::path::{Path, PathBuf};

/// Converts a string into a path buffer.
///
/// Arguments:
///
/// * `path` - String
///
/// Usage: ```get_file_path(path);```
/// notes: using AsRef creates a cheap reference-to-reference conversion
pub fn get_root_dir() -> &'static str {
  Path::new(relative!("static")).to_str().unwrap()
}

/// Converts a string into a path buffer.
///
/// Arguments:
///
/// * `path` - String
///
/// Usage: ```get_file_path(path);```
/// notes: using AsRef creates a cheap reference-to-reference conversion
pub fn get_file_path(path: impl AsRef<Path>) -> PathBuf {
  Path::new(relative!("static")).join(path)
}

// OLD:
// pub fn get_file_path(path: String) -> PathBuf {
//   Path::new(relative!("static")).join(path.clone())
// }

/// Converts a path buffer into a string.
///
/// Arguments:
///
/// * `path` - PathBuf
///
/// Usage: ```get_string_path(path);```
/// notes: using AsRef creates a cheap reference-to-reference conversion
/// to_str: yields a &str slice if the OsStr is valid Unicode.
/// into: value-to-value conversion that consumes the input value
pub fn get_string_path(path: impl AsRef<Path>) -> String {
  path.as_ref().to_str().unwrap().into()
}

// OLD:
// pub fn get_string_path(path: PathBuf) -> String {
//   path.into_os_string().into_string().unwrap()
// }

// Retrieves a file extension from a string.
//
// Arguments:
//
// * `filename` - &str
//
// Usage: get_extension_from_filename(&filename);
//
// pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
//   match Path::new(filename).extension().and_then(OsStr::to_str) {
//     Some(ext) => Some(ext),
//     None => None,
//   }
// }
