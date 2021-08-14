// #![allow(dead_code, unused_variables)]

use crate::lrucache::LRUCache;
use crate::utils::{get_extension_from_filename, get_file_path, get_string_path};
use futures_locks::Mutex;
use image::imageops::FilterType;
use regex::Regex;
use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer};
use rocket::http::ContentType;
use rocket::response::content::Custom;
use rocket::State;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

type Cache = Mutex<LRUCache<String, Vec<u8>>>;

type CustomResponseVector = Custom<Vec<u8>>;

// API endpoint: GET http://domain.com/image/example.png?width=800
// Try visiting:
//   http://127.0.0.1:5000/placeholder.png
//   http://127.0.0.1:5000/placeholder.png?width=800
//   http://127.0.0.1:5000/placeholder.png?width=1000
//   http://127.0.0.1:5000/placeholder_500.png?width=1000 => falls back to placeholder.png@1000px
#[get("/image/<path..>?<width>")]
async fn serve_image(
  path: PathBuf,
  width: Option<&str>,
  state: &State<Cache>,
) -> Option<CustomResponseVector> {
  // convert path to string -> <filename>.<ext>
  let fp = get_string_path(path.to_path_buf());

  // if present, strip any included "_<width>" from the filename
  let filename = Regex::new(r"_.*[\d]")
    .unwrap()
    .replace_all(&fp, "")
    .to_string();

  // retrieve static folder + filename -> /path/to/static/<filename> buffer
  let fn_path = get_file_path(filename);

  // derive content type from filename -> example.png -> .png -> ContentType::PNG
  let content_type = match get_extension_from_filename(&fp) {
    Some(ext) => ContentType::from_extension(ext),
    None => None,
  };

  // fallback to 404 route if path is empty or file extension is invalid
  if path.as_os_str().is_empty() || content_type.is_none() {
    rocket::info_!("The file path and/or content type are invalid.");
    return None;
  }
  let mut cache = state.lock().await;

  // retrieve string of static folder with filename -> /path/to/static/<filename>.<ext>
  let pathname = get_string_path(fn_path.to_path_buf());

  // TODO - Create standardized widths to prevent unlimited amount of image resizes
  // converts supplied "width" argument to a valid u32
  let parsed_width = width.unwrap_or("0").parse::<u32>().unwrap_or(0);

  // store original pathname if no width query or store <filename>_<width>.<ext>
  let requested_fp = match parsed_width == 0 {
    true => pathname,
    false => {
      let image_path = get_string_path(fn_path.to_path_buf());

      let new_image_path: Vec<&str> = image_path.split('.').collect();

      format!(
        "{}_{}.{}",
        &new_image_path[0],
        &parsed_width.to_string(),
        &new_image_path[1],
      )
    }
  };

  // determine if cache contains requested image
  match cache.contains_key(&requested_fp) {
    true => {
      rocket::info_!("Served requested image from cache.");

      // retrieve image from the cache
      let stored_entry = match cache.get(&requested_fp.to_string()) {
        Some(val) => val,
        None => panic!("Unable to retrieve image entry from cache."),
      };

      // respond to request with cached image
      Some(Custom(content_type.unwrap(), stored_entry.to_vec()))
    }
    false => {
      // return if requested image doesn't exist
      if !fn_path.is_file() {
        return None;
      }

      // TODO - Make sure requested image size doesn't extend beyond actual image dimensions
      // create a new image from original if one doesn't exist
      if !get_file_path(requested_fp.to_string()).is_file() {
        let current_image = image::open(fn_path).expect("Failed to open image.");
        current_image
          .resize(parsed_width, parsed_width, FilterType::CatmullRom)
          .save(&requested_fp)
          .expect("Failed to resize image.");
      }

      // open requested image
      let mut named_file = match File::open(&requested_fp).await {
        Ok(file) => file,
        Err(_err) => panic!("Unable to open image."),
      };

      // read the contents of the image
      let mut contents = Vec::new();
      match named_file.read_to_end(&mut contents).await {
        Ok(vec) => vec,
        Err(_f) => panic!("Unable to read the contents of the image."),
      };

      // store read contents into cache
      cache.insert(requested_fp.to_string(), contents);

      // retrieve contents from the cache
      let stored_entry = match cache.get(&requested_fp.to_string()) {
        Some(val) => val,
        None => panic!("Unable to retrieve entry from cache."),
      };

      rocket::info_!("Saved requested image into cache.");

      // respond to request with (original/resized) image
      Some(Custom(content_type.unwrap(), stored_entry.to_vec()))
    }
  }
}

pub fn stage() -> AdHoc {
  AdHoc::on_ignite("serve", |rocket| async {
    rocket
      .mount("/", routes![serve_image])
      .mount("/", FileServer::from(relative!("static")))
      .manage(Mutex::new(LRUCache::<String, Vec<u8>>::new(1000)))
  })
}
