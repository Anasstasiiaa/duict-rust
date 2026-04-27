use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::DynamicImage;

mod fs_uploader;
mod s3_uploader;
mod uploader;

use fs_uploader::FsUploader;
use s3_uploader::S3Uploader;
use uploader::Uploader;

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

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let config = parse_args(&args)?;

    let uploader = get_uploader();

    let file = File::open(&config.list_path)
        .map_err(|e| format!("Не вдається відкрити файл зі списком: {e}"))?;
    let reader = BufReader::new(file);

    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Помилка читання рядка {idx}: {e}"))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Err(err) = process_entry(trimmed, &config, idx, uploader.as_ref()) {
            eprintln!("Помилка для запису #{idx} (`{trimmed}`): {err}");
        }
    }

    Ok(())
}

fn get_uploader() -> Box<dyn Uploader> {
    match env::var("MYME_UPLOADER")
        .unwrap_or("fs".to_string())
        .as_str()
    {
        "s3" => Box::new(S3Uploader {
            bucket: env::var("S3_BUCKET").expect("S3_BUCKET not set"),
        }),
        _ => Box::new(FsUploader),
    }
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
                    return Err("Прапор --files потребує шлях до файлу".into());
                }
                list_path = Some(PathBuf::from(&args[i]));
            }
            "--resize" => {
                i += 1;
                if i >= args.len() {
                    return Err("Прапор --resize потребує значення widthxheight".into());
                }
                resize = Some(parse_resize(&args[i])?);
            }
            other => {
                return Err(format!("Невідомий аргумент: {other}"));
            }
        }
        i += 1;
    }

    let list_path = list_path.ok_or_else(|| "Не вказано файл зі списком (--files)".to_string())?;
    let (width, height) =
        resize.ok_or_else(|| "Не вказано розмір (--resize widthxheight)".to_string())?;

    Ok(Config {
        list_path,
        width,
        height,
    })
}

fn parse_resize(s: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
        return Err("Формат --resize: widthxheight".into());
    }

    let width: u32 = parts[0].parse().map_err(|_| "width не число")?;
    let height: u32 = parts[1].parse().map_err(|_| "height не число")?;

    Ok((width, height))
}

fn process_entry(
    entry: &str,
    config: &Config,
    index: usize,
    uploader: &dyn Uploader,
) -> Result<(), String> {
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
        .map_err(|e| format!("Помилка кодування: {e}"))?;

    uploader.upload(&filename, buffer);

    println!("Оброблено: {}", filename);

    Ok(())
}

fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

fn load_image_from_url(url: &str) -> Result<DynamicImage, String> {
    let response = reqwest::blocking::get(url).map_err(|e| format!("HTTP помилка: {e}"))?;

    let bytes = response.bytes().map_err(|e| format!("Bytes error: {e}"))?;

    image::load_from_memory(&bytes).map_err(|e| format!("Image decode error: {e}"))
}

fn load_image_from_path(path: &str) -> Result<DynamicImage, String> {
    image::open(path).map_err(|e| format!("File error: {e}"))
}

fn resize_image(img: DynamicImage, width: u32, height: u32) -> DynamicImage {
    img.resize_exact(width, height, FilterType::Lanczos3)
}

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
