// #![allow(dead_code, unused_variables)]

use crate::lrucache::LRUCache;
use crate::reqimage::RequestedImage;
use crate::utils::{send_400_response, send_404_response, InvalidRequest};
use futures_locks::Mutex;
use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer};
use rocket::response::content::Custom;
use rocket::State;
use std::path::PathBuf;

type Cache = Mutex<LRUCache<String, Vec<u8>>>;

type ResVec = Custom<Vec<u8>>;

#[get("/image/<path..>?<width>")]
async fn serve_image(
    path: PathBuf,
    width: Option<&str>,
    state: &State<Cache>,
) -> Result<ResVec, InvalidRequest> {
    // ensure that path is a directory
    if path.extension().is_none() || path.as_os_str().is_empty() {
        return Err(send_404_response("The file path is invalid.".to_string()));
    }

    // initialize requested image
    let req_image = RequestedImage::new(&path, width);

    // ensure the requested image has a valid content type
    if req_image.content_type.is_none() {
        return Err(send_400_response(
            "The image content type is invalid.".to_string(),
        ));
    }

    let mut cache = state.lock().await;
    // determine if cache contains requested image
    if !cache.contains_key(&req_image.new_pathname) {
        // return if requested image doesn't exist
        if !req_image.path.is_file() {
            return Err(send_404_response("Resource was not found.".to_string()));
        }

        // create a new image from original if one doesn't exist already
        if !req_image.exists() {
            match req_image.save() {
                Ok(()) => (),
                Err(reason) => return Err(send_400_response(reason.to_string())),
            };
        }

        // read the original or new image and store its contents into cache
        match req_image.read().await {
            Ok(contents) => cache.insert(req_image.new_pathname.clone(), contents),
            Err(reason) => return Err(send_400_response(reason.to_string())),
        };

        info_!("Saved requested image into cache.");
    }

    // retrieve saved image from the cache
    let cached_image = cache
        .get(&req_image.new_pathname)
        .expect("Unable to retrieve image entry from cache.");

    info_!("Served requested image from cache.");

    // respond with cached image
    Ok(Custom(
        req_image.content_type.unwrap(),
        cached_image.to_vec(),
    ))
}

pub fn main() -> AdHoc {
    AdHoc::on_ignite("serve", |rocket| async {
        rocket
            .mount("/", routes![serve_image])
            .mount("/", FileServer::from(relative!("static")))
            .manage(Mutex::new(LRUCache::<String, Vec<u8>>::new(50)))
    })
}
