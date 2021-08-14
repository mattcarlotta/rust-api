// #![allow(dead_code, unused_variables)]

use crate::lrucache::LRUCache;
use crate::utils::{get_file_path, get_string_path};
use futures_locks::Mutex;
use image::imageops::FilterType;
use regex::Regex;
use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer, NamedFile};
use rocket::State;
use std::path::PathBuf;

type Cache = Mutex<LRUCache<String, String>>;

#[get("/image/<path..>?<width>")]
async fn serve_image(
  path: PathBuf,
  width: Option<&str>,
  state: &State<Cache>,
) -> Option<NamedFile> {
  // prevent width reduplication when path includes resized widths
  // so that <filename>_<width>.<ext> becomes <filename>.<ext>
  let filename = Regex::new(r"_.*[\d]")
    .unwrap()
    .replace_all(&get_string_path(path.clone()), "")
    .to_string();

  let fn_path = get_file_path(filename);

  // return if path is empty or file doesn't exist
  if path.as_os_str().is_empty() || !fn_path.is_file() {
    return None;
  }
  let mut cache = state.lock().await;

  let pathname = get_string_path(fn_path.clone());

  // TODO - Create standardized widths to prevent unlimited amount of image resizes
  // convert width to u32
  let parsed_width = width.unwrap_or("0").parse::<u32>().unwrap_or(0);

  let requested_fp = match parsed_width == 0 {
    // return original pathname if no width query
    true => pathname,
    // or return filename_width.ext
    false => {
      let image_path = get_string_path(fn_path.clone());

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

  if !cache.contains_key(&requested_fp) {
    // check if a resized image of the original exists: "original_width.ext"
    if !get_file_path(requested_fp.to_string()).is_file() {
      let current_image = image::open(fn_path).expect("Failed to open file.");

      // resize and save image to new width
      current_image
        .resize(parsed_width, parsed_width, FilterType::CatmullRom)
        .save(&requested_fp)
        .expect("Failed to resize file.");
    }

    // insert filename into cache
    cache.insert(requested_fp.to_string(), requested_fp.to_string());
  }

  NamedFile::open(requested_fp).await.ok()
}

pub fn stage() -> AdHoc {
  AdHoc::on_ignite("serve", |rocket| async {
    rocket
      .mount("/", routes![serve_image])
      .mount("/", FileServer::from(relative!("static")))
      .manage(Mutex::new(LRUCache::<String, String>::new(1000)))
  })
}
