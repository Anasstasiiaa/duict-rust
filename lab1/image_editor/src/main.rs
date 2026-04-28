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
use std::time::{SystemTime, UNIX_EPOCH};

use image::imageops::FilterType;
use image::DynamicImage;

/// Configuration parsed from CLI arguments and environment.
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

    let file = File::open(&config.list_path)
        .map_err(|e| format!("Не вдається відкрити файл зі списком: {e}"))?;

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

fn parse_args(args: &[String]) -> Result<Config, String> {
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
            other => return Err(format!("Невідомий аргумент: {other}")),
        }
        i += 1;
    }

    let list_path = list_path.ok_or_else(|| "Не вказано файл зі списком (--files)".to_string())?;

    let (width, height) =
        resize.ok_or_else(|| "Не вказано розмір (--resize widthxheight)".to_string())?;

    // FIX ISSUE #1 — нормальна обробка env без panic
    let output_dir_str =
        env::var("MYME_FILES_PATH").map_err(|_| "MYME_FILES_PATH не встановлено".to_string())?;

    let output_dir = PathBuf::from(output_dir_str);

    if !output_dir.exists() {
        return Err(format!("Каталог не існує: {}", output_dir.display()));
    }

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
        return Err("Формат --resize: widthxheight".into());
    }

    let width: u32 = parts[0].parse().map_err(|_| "width не число")?;
    let height: u32 = parts[1].parse().map_err(|_| "height не число")?;

    if width == 0 || height == 0 {
        return Err("Розмір має бути > 0".into());
    }

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

    let output_path = build_output_path(
        entry,
        &config.output_dir,
        config.width,
        config.height,
        index,
    );

    resized
        .save(&output_path)
        .map_err(|e| format!("Помилка збереження: {e}"))?;

    Ok(())
}

/// Check if string is URL.
fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

fn load_image_from_url(url: &str) -> Result<DynamicImage, String> {
    let response = reqwest::blocking::get(url).map_err(|e| format!("HTTP помилка: {e}"))?;

    let bytes = response.bytes().map_err(|e| format!("Bytes error: {e}"))?;

    image::load_from_memory(&bytes).map_err(|e| format!("Decode error: {e}"))
}

fn load_image_from_path(path: &str) -> Result<DynamicImage, String> {
    image::open(path).map_err(|e| format!("File error: {e}"))
}

/// Resize image.
fn resize_image(img: DynamicImage, width: u32, height: u32) -> DynamicImage {
    img.resize_exact(width, height, FilterType::Lanczos3)
}

fn build_output_path(
    original: &str,
    output_dir: &Path,
    width: u32,
    height: u32,
    index: usize,
) -> PathBuf {
    let base = if is_url(original) {
        original.split('/').last().unwrap_or("image")
    } else {
        Path::new(original)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("image")
    };

    let name = base.split('.').next().unwrap_or("image");

    // FIX ISSUE #2 — унікальність через timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let file_name = format!("{name}_{width}x{height}_{index}_{timestamp}.png");

    output_dir.join(file_name)
}
