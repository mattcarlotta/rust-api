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

#[get("/image/<path..>?<width>")]
async fn serve_image(
  path: PathBuf,
  width: Option<&str>,
  state: &State<Cache>,
) -> Option<CustomResponseVector> {
  // prevent width reduplication when path includes resized widths
  // so that <filename>_<width>.<ext> becomes <filename>.<ext>
  let fp = get_string_path(path.clone());

  // strip any "_<width>" file paths
  let filename = Regex::new(r"_.*[\d]")
    .unwrap()
    .replace_all(&fp, "")
    .to_string();

  // retrieve static folder + filename -> /path/to/static/<filename> buffer
  let fn_path = get_file_path(filename);

  // get content type
  let content_type = match get_extension_from_filename(&fp) {
    Some(ext) => ContentType::from_extension(ext),
    None => None,
  };

  // return if path is empty or file extension is invalid
  if path.as_os_str().is_empty() || content_type.is_none() {
    rocket::info_!("The file path and/or content type are invalid.");
    return None;
  }
  let mut cache = state.lock().await;

  // retrieve static folder + filename -> /path/to/static/<filename> string
  let pathname = get_string_path(fn_path.clone());

  // TODO - Create standardized widths to prevent unlimited amount of image resizes
  // TODO - Make sure requested image size doesn't extend beyond actual image dimension
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

  // determine if cache contains requested file
  match cache.contains_key(&requested_fp) {
    true => {
      rocket::info_!("Served requested file from cache.");

      Some(Custom(
        content_type.unwrap(),
        cache.get(&requested_fp).unwrap().clone(),
      ))
    }
    false => {
      // return if file doesn't exist
      if !fn_path.is_file() {
        return None;
      }
      // check if a resized image of the original exists: "original_width.ext"
      if !get_file_path(requested_fp.to_string()).is_file() {
        let current_image = image::open(fn_path).expect("Failed to open file.");
        // resize and save image to new width
        current_image
          .resize(parsed_width, parsed_width, FilterType::CatmullRom)
          .save(&requested_fp)
          .expect("Failed to resize file.");
      }

      // open requested file
      let mut named_file = File::open(&requested_fp).await.ok().unwrap();

      // read the contents
      let mut contents = Vec::new();
      named_file.read_to_end(&mut contents).await.ok();

      // store contents into cache
      cache.insert(requested_fp.to_string(), contents.clone());

      rocket::info_!("Saved requested file into cache.");

      Some(Custom(content_type.unwrap(), contents))
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
