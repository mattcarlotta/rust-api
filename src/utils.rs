use rocket::fs::relative;
use rocket::response::content::Html;
use rocket::response::status::{BadRequest, NotFound};
use std::path::{Path, PathBuf};

#[derive(Debug, Responder)]
pub enum InvalidRequest {
    NotFnd(NotFound<Html<String>>),
    BadReq(BadRequest<String>),
}

/// Converts a string into a path buffer.
///
/// Arguments:
///
/// * `path` - String
///
/// Returns: `&'static str`
///
/// Usage: ```get_file_path(path);```
pub fn get_root_dir() -> &'static str {
    Path::new(relative!("static")).to_str().unwrap()
}

/// Joins a pathbuf with a relative path to the `static` folder.
///
/// Arguments:
///
/// * `path` - String
///
/// Returns: `PathBuf`
///
/// Usage: ```get_file_path(path);```
pub fn get_file_path(path: impl AsRef<Path>) -> PathBuf {
    Path::new(relative!("static")).join(path)
}

/// Converts a path buffer into a string.
///
/// Arguments:
///
/// * `path` - PathBuf
///
/// Returns: `String`
///
/// Usage: ```get_string_path(path);```
pub fn get_string_path(path: impl AsRef<Path>) -> String {
    path.as_ref().to_str().unwrap().into()
}

/// Determines standardized ratios
/// exptects: 0, 20, 35, 50, 75, 90
///
/// Arguments:
///
/// * `ratio` - u8
///
/// Returns `bool`
///
/// Usage: ```non_standardized(ratio);
pub fn non_standardized(r: u8) -> bool {
    match r {
        0 => return false,
        20 => return false,
        35 => return false,
        50 => return false,
        75 => return false,
        90 => return false,
        _ => return true,
    }
}

/// Reusable 400 response.
///
/// Arguments:
///
/// * `reason` - String
///
/// Returns: `BadRequest<Option<String>>`
///
/// Usage: ```send_error_response(reason);```
pub fn send_400_response(reason: String) -> InvalidRequest {
    InvalidRequest::BadReq(BadRequest(Some(reason)))
}

/// Reusable 404 response.
///
/// Arguments:
///
/// * `reason` - &str
///
/// Returns: `NotFound(Html<String>)`
///
/// Usage: ```send_error_response(reason);```
pub fn send_404_response(reason: String) -> InvalidRequest {
    InvalidRequest::NotFnd(NotFound(Html(format!(
    "<!DOCTYPE html><html lang='en' style='height: 100%;'><head><meta charset='utf-8'><title>Resource Not Found</title></head><body style='height: 100%;margin: 0;'><div style='display: -webkit-box;display: -ms-flexbox;display: flex;-webkit-box-orient: vertical;-webkit-box-direction: normal;-ms-flex-direction: column;flex-direction: column;-webkit-box-pack: center;-ms-flex-pack: center;justify-content: center;height: 100%;'><h1 style='text-align:center;font-size:100px;margin:0;'>404 Not Found</h1><h1 style='text-align:center;'>{}</h1></body></div></html>",
    reason
  ))))
}
