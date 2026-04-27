//! Image editor CLI tool.
//!
//! Reads images from file/URL, resizes them and uploads
//! either to filesystem or S3 depending on configuration.

#![deny(missing_docs)]
#![deny(missing_crate_level_docs)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::result_large_err)]

mod errors;
use errors::AppError;

mod fs_uploader;
mod s3_uploader;
mod uploader;

use fs_uploader::FsUploader;
use s3_uploader::S3Uploader;
use uploader::Uploader;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::DynamicImage;

/// Configuration parsed from CLI arguments.
struct Config {
    list_path: PathBuf,
    width: u32,
    height: u32,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Entry point of application.
///
/// # Errors
/// Returns [`AppError`] if any IO, image or configuration error occurs.
fn run() -> Result<(), AppError> {
    let args: Vec<String> = env::args().collect();
    let config = parse_args(&args)?;

    let uploader = get_uploader();

    let file = File::open(&config.list_path)?;
    let reader = BufReader::new(file);

    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if let Err(err) = process_entry(trimmed, &config, idx, uploader.as_ref()) {
            eprintln!("Error processing #{idx} (`{trimmed}`): {err}");
        }
    }

    Ok(())
}

/// Select uploader based on environment variable.
///
/// Defaults to filesystem uploader.
fn get_uploader() -> Box<dyn Uploader> {
    match env::var("MYME_UPLOADER")
        .unwrap_or_else(|_| "fs".to_string())
        .as_str()
    {
        "s3" => Box::new(S3Uploader {
            bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "default-bucket".to_string()),
        }),
        _ => Box::new(FsUploader),
    }
}

/// Parse CLI arguments.
///
/// # Errors
/// Returns error if arguments are missing or invalid.
fn parse_args(args: &[String]) -> Result<Config, AppError> {
    let mut list_path: Option<PathBuf> = None;
    let mut resize: Option<(u32, u32)> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--files" => {
                i += 1;
                if i >= args.len() {
                    return Err(AppError::Config("--files requires path".into()));
                }
                list_path = Some(PathBuf::from(&args[i]));
            }
            "--resize" => {
                i += 1;
                if i >= args.len() {
                    return Err(AppError::Config("--resize requires widthxheight".into()));
                }
                resize = Some(parse_resize(&args[i])?);
            }
            other => {
                return Err(AppError::Config(format!("Unknown arg: {other}")));
            }
        }
        i += 1;
    }

    let list_path = list_path.ok_or_else(|| AppError::Config("Missing --files argument".into()))?;
    let (width, height) =
        resize.ok_or_else(|| AppError::Config("Missing --resize argument".into()))?;

    Ok(Config {
        list_path,
        width,
        height,
    })
}

/// Parse resize argument.
fn parse_resize(s: &str) -> Result<(u32, u32), AppError> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
        return Err(AppError::Config(
            "Invalid format, expected widthxheight".into(),
        ));
    }

    let width: u32 = parts[0]
        .parse()
        .map_err(|_| AppError::Config("Width must be a positive number".into()))?;

    let height: u32 = parts[1]
        .parse()
        .map_err(|_| AppError::Config("Height must be a positive number".into()))?;

    Ok((width, height))
}

/// Process single image entry.
fn process_entry(
    entry: &str,
    config: &Config,
    index: usize,
    uploader: &dyn Uploader,
) -> Result<(), AppError> {
    let img = if is_url(entry) {
        load_image_from_url(entry)?
    } else {
        load_image_from_path(entry)?
    };

    let resized = resize_image(img, config.width, config.height);

    let filename = build_filename(entry, config.width, config.height, index);

    let mut buffer = Vec::new();

    resized
        .write_to(
            &mut std::io::Cursor::new(&mut buffer),
            image::ImageFormat::Png,
        )
        .map_err(AppError::Image)?;

    uploader.upload(&filename, buffer)?;

    println!("Processed: {filename}");

    Ok(())
}

/// Check if string is URL.
fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

/// Load image from URL.
fn load_image_from_url(url: &str) -> Result<DynamicImage, AppError> {
    let response = reqwest::blocking::get(url).map_err(|e| AppError::Http(e.to_string()))?;

    let bytes = response
        .bytes()
        .map_err(|e| AppError::Http(e.to_string()))?;

    image::load_from_memory(&bytes).map_err(AppError::Image)
}

/// Load image from file path.
fn load_image_from_path(path: &str) -> Result<DynamicImage, AppError> {
    image::open(path).map_err(AppError::Image)
}

/// Resize image.
fn resize_image(img: DynamicImage, width: u32, height: u32) -> DynamicImage {
    img.resize_exact(width, height, FilterType::Lanczos3)
}

/// Build output filename.
fn build_filename(original: &str, width: u32, height: u32, index: usize) -> String {
    let base = if is_url(original) {
        original.split('/').last().unwrap_or("image")
    } else {
        Path::new(original)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("image")
    };

    let name = base.split('.').next().unwrap_or("image");

    format!("{name}_{width}x{height}_{index}.png")
}
