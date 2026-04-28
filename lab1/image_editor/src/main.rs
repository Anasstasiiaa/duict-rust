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
use std::path::{Path, PathBuf};
use std::time::Instant;

use image::imageops::FilterType;
use image::DynamicImage;

use rayon::prelude::*;
use tokio::fs;
use tokio::runtime::Builder;

/// Config
struct Config {
    list_path: PathBuf,
    width: u32,
    height: u32,
}

fn main() {
    // ⏱ benchmark start
    let start = Instant::now();

    // ⚙️ MANUAL TOKIO RUNTIME CONFIGURATION (ЛР8 requirement)
    let runtime = Builder::new_multi_thread()
        .worker_threads(num_cpus::get()) // оптимально = CPU cores
        .max_blocking_threads(8)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        if let Err(e) = run().await {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    });

    // ⏱ benchmark end
    println!("Time: {:?}", start.elapsed());
}

async fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let config = parse_args(&args)?;

    // 📥 ASYNC IO (file read)
    let content = fs::read_to_string(&config.list_path)
        .await
        .map_err(|e| format!("Cannot read file: {e}"))?;

    let entries: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // 🧠 CPU-bound parallel processing
    entries.par_iter().enumerate().for_each(|(idx, entry)| {
        if entry.trim().is_empty() {
            return;
        }

        if let Err(err) = process_entry(entry, &config, idx) {
            eprintln!("Error #{idx} ({entry}): {err}");
        }
    });

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
                list_path = Some(PathBuf::from(&args[i]));
            }
            "--resize" => {
                i += 1;
                resize = Some(parse_resize(&args[i])?);
            }
            other => return Err(format!("Unknown arg: {other}")),
        }
        i += 1;
    }

    let list_path = list_path.ok_or_else(|| "Missing --files".to_string())?;
    let (width, height) = resize.ok_or_else(|| "Missing --resize".to_string())?;

    let output_dir = PathBuf::from(
        env::var("MYME_FILES_PATH")
            .map_err(|_| "MYME_FILES_PATH not set".to_string())?,
    );

    if !output_dir.exists() {
        return Err("Output dir does not exist".to_string());
    }

    Ok(Config {
        list_path,
        width,
        height,
    })
}

fn parse_resize(s: &str) -> Result<(u32, u32), String> {
    if !s.contains('x') {
        return Err("Invalid format (expected widthxheight)".into());
    }

    let parts: Vec<&str> = s.split('x').collect();

    let width: u32 = parts[0].parse().map_err(|_| "bad width")?;
    let height: u32 = parts[1].parse().map_err(|_| "bad height")?;

    Ok((width, height))
}

fn process_entry(entry: &str, config: &Config, index: usize) -> Result<(), String> {
    let img = load_image_from_path(entry)?;

    let resized = resize_image(img, config.width, config.height);

    let output_path =
        build_output_path(entry, &config.output_dir, config.width, config.height, index);

    resized
        .save(&output_path)
        .map_err(|e| format!("save error: {e}"))?;

    println!("Saved: {}", output_path.display());
    Ok(())
}

fn load_image_from_path(path: &str) -> Result<DynamicImage, String> {
    image::open(path).map_err(|e| format!("file error: {e}"))
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
    let base = Path::new(original)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let name = base.split('.').next().unwrap_or("image");

    output_dir.join(format!("{name}_{width}x{height}_{index}.png"))
}
