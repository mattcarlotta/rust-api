// #![allow(dead_code, unused_variables)]

use image::imageops::FilterType;
use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer, NamedFile};
use std::path::{Path, PathBuf};

/// Converts a string into a path buffer.
///
/// Arguments:
///
/// * `path` - &String
///
/// Usage: ```get_file_path(path);```
fn get_file_path(path: &String) -> PathBuf {
  Path::new(relative!("static")).join(path)
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
async fn serve_image(path: PathBuf, width: Option<&str>) -> Option<NamedFile> {
  // return if path is empty
  if path.as_os_str().is_empty() {
    return None;
  }

  // get file path buffer
  let file_path = get_file_path(&get_string_path(path));

  // TODO - Sanitize widths to prevent n amount of image resizes
  // convert width to u32
  let parsed_width = width.unwrap_or("0").parse::<u32>().unwrap_or(0);

  if file_path.is_file() && parsed_width > 0 {
    let image_path = get_string_path(file_path).clone();

    // split string by "." => filepath.ext => (filepath, ext)
    let path_vector: Vec<&str> = image_path.split('.').collect();

    // join width with file name and ext => filename_width.ext
    let resized_fp = [
      path_vector[0],
      "_",
      &parsed_width.to_string(),
      ".",
      path_vector[1],
    ]
    .join("");

    // create a resized image if the "image_width.ext" file doesn't exist
    if !get_file_path(&resized_fp).is_file() {
      let current_image = image::open(image_path).expect("Failed to open file.");

      // resize image by width and save new image file
      current_image
        .resize(parsed_width, parsed_width, FilterType::CatmullRom)
        .save(&resized_fp)
        .expect("Failed to resize file.");
    }

    return NamedFile::open(resized_fp).await.ok();
  }

  return NamedFile::open(file_path).await.ok();
}

pub fn stage() -> rocket::fairing::AdHoc {
  AdHoc::on_ignite("serve", |rocket| async {
    rocket
      .mount("/", routes![serve_image])
      .mount("/", FileServer::from(relative!("static")))
  })
}
