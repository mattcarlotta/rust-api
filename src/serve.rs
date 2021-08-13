// #![allow(dead_code, unused_variables)]

use crate::lrucache::LRUCache;
use futures_locks::Mutex;
use image::imageops::FilterType;
use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer, NamedFile};
use rocket::State;
use std::path::{Path, PathBuf};

/// Converts a string into a path buffer.
///
/// Arguments:
///
/// * `path` - String
///
/// Usage: ```get_file_path(path);```
fn get_file_path(path: String) -> PathBuf {
  Path::new(relative!("static")).join(path.clone())
}

/// Converts a path buffer into a string.
///
/// Arguments:
///
/// * `path` - PathBuf
///
/// Usage: ```get_string_path(path);```
fn get_string_path(path: PathBuf) -> String {
  path.into_os_string().into_string().unwrap()
}

#[get("/image/<path..>?<width>")]
async fn serve_image(
  path: PathBuf,
  width: Option<&str>,
  state: &State<Mutex<LRUCache<String, String>>>,
) -> Option<NamedFile> {
  // return if path is empty
  if path.as_os_str().is_empty() {
    return None;
  }
  let mut cache = state.lock().await;

  let fp = get_file_path(get_string_path(path));
  let pathname = get_string_path(fp.clone());

  // TODO - Create standardized widths to prevent unlimited amount of image resizes
  // convert width to u32
  let parsed_width = width.unwrap_or("0").parse::<u32>().unwrap_or(0);

  // determine whether or not the requested file contains a width
  let requested_fp = match parsed_width == 0 {
    // and return original pathname
    true => pathname,
    // or return filename_width.ext
    false => {
      let image_path = get_string_path(fp.clone());

      // split string by "." => filepath.ext => (filepath, ext)
      let new_image_path: Vec<&str> = image_path.split('.').collect();

      // join width with file name and ext => filename_width.ext
      format!(
        "{}_{}.{}",
        &new_image_path[0],
        &parsed_width.to_string(),
        &new_image_path[1],
      )
    }
  };

  // determine if cache contains requested file
  let file: String = match cache.contains_key(&requested_fp) {
    true => requested_fp,
    false => {
      // check if a resized image of the original exists: "original_width.ext"
      if !get_file_path(requested_fp.to_string()).is_file() {
        // open the image
        let current_image = image::open(fp).expect("Failed to open file.");

        // resize and save image to new width
        current_image
          .resize(parsed_width, parsed_width, FilterType::CatmullRom)
          .save(&requested_fp)
          .expect("Failed to resize file.");
      }

      // insert file into cache
      cache.insert(requested_fp.to_string(), requested_fp.to_string());

      requested_fp
    }
  };

  NamedFile::open(file).await.ok()
}

pub fn stage() -> AdHoc {
  AdHoc::on_ignite("serve", |rocket| async {
    let cache = Mutex::new(LRUCache::<String, String>::new(20));
    rocket
      .mount("/", routes![serve_image])
      .mount("/", FileServer::from(relative!("static")))
      .manage(cache)
  })
}
