use crate::utils::{get_file_path, get_root_dir, get_string_path};
use image::imageops::FilterType;
use image::GenericImageView;
use regex::Regex;
use rocket::http::ContentType;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug)]
pub struct RequestedImage {
    pub content_type: Option<ContentType>,
    pub path: PathBuf,
    pub new_pathname: String,
    pub new_pathname_buf: PathBuf,
    pub width: u32,
}

impl<'p, 'w> RequestedImage {
    /// Initialize a new requested image that:
    /// * strips out any provided widths within the stem -> filename_width -> filename
    /// * creates buffers from the stripped pathname and a potential new path (filename_width.ext)
    /// * retrieves content type from requested image
    ///
    /// Arguments:
    ///
    /// * `path` - PathBuf
    /// * `width` - Option<&str>
    ///
    /// Usage: ```RequestedImage::new(&path, width);```
    pub fn new(path: &'p PathBuf, width: Option<&'w str>) -> Self {
        // if present, strip any included "_<width>" from the filename
        let filename = Regex::new(r"_.*[\d]")
            .unwrap()
            .replace_all(&get_string_path(path.to_path_buf()), "")
            .to_string();

        // retrieve file path to "static" folder => <rootdir><static><filename>.<ext>
        let filepath = get_file_path(filename);

        // TODO - Create standardized widths to prevent unlimited amount of image resizes
        // converts supplied "width" argument to a valid u32
        let width = width
            .map(str::parse::<u32>)
            .map(Result::ok)
            .flatten()
            .unwrap_or(0);

        // assign original pathname if no width query: <rootdir><filename>.<ext>
        // or assign pathname with width: <rootdir><filename>_<width>.<ext>
        let pathname = match width == 0 {
            true => get_string_path(&filepath),
            false => {
                // retrieve image file stem => <filename>
                let stem = &filepath
                    .file_stem()
                    .and_then(OsStr::to_str)
                    .expect(&format!("Image is missing stem"));

                // retrieve image file stem => <ext>
                let ext = &filepath
                    .extension()
                    .and_then(OsStr::to_str)
                    .expect(&format!("Image is missing extension"));
                format!("{}/{}_{}.{}", get_root_dir(), stem, width, ext)
            }
        };

        RequestedImage {
            content_type: path
                .extension()
                .and_then(OsStr::to_str)
                .and_then(ContentType::from_extension),
            path: get_file_path(&filepath),
            new_pathname: pathname.to_string(),
            new_pathname_buf: Path::new(&pathname).to_path_buf(),
            width,
        }
    }

    /// Determines if a requested image path with width already exists
    ///
    /// Arguments: (none)
    ///
    /// Usage: ```req_image.exists();```
    pub fn exists(&self) -> bool {
        self.new_pathname_buf.is_file()
    }

    /// Saves a new image to disk with the provided resized width of the requested image
    ///
    /// Arguments: (none)
    ///
    /// Usage: ```req_image.save();```
    pub fn save(&self) -> Result<(), String> {
        // open original image
        let original_image = image::open(&self.path).expect("Failed to open image.");

        let (width, ..) = original_image.dimensions();

        if self.width >= width {
            return Err(format!("Unable to request a width of {}px because it meets or exceeds the original image's width of {}px.", self.width, width).to_string());
        }

        // resize and save it as the requested width
        original_image
            .resize(self.width, self.width, FilterType::CatmullRom)
            .save(self.new_pathname.to_string())
            .expect("Failed to resize image.");

        Ok(())
    }

    /// Asynchronously reads the requested image and returns its contents as `Vec<u8>`
    ///
    /// Arguments: (none)
    ///
    /// Usage: ```req_image.read();```
    pub async fn read(&self) -> Result<Vec<u8>, String> {
        // TODO - Make sure requested image size doesn'p extend beyond actual image dimensions
        // open requested image
        let mut existing_file = match File::open(&self.new_pathname).await {
            Ok(file) => file,
            Err(reason) => {
                return Err(format!("Unable to open image: {}", reason).to_string());
            }
        };

        // read the contents of the image
        let mut contents = Vec::new();
        match existing_file.read_to_end(&mut contents).await {
            Ok(vec) => vec,
            Err(reason) => {
                rocket::info_!("Unable to read the contents of the image: {}", reason);
                return Err("Resource was not found.".to_string());
            }
        };

        Ok(contents)
    }
}
