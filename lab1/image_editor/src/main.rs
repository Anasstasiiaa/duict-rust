use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;

use image::imageops::FilterType;
use image::DynamicImage;

use futures::future::join_all;
use tokio::fs;
use tokio::runtime::Builder;

/// Config
#[derive(Clone)]
struct Config {
    list_path: PathBuf,
    width: u32,
    height: u32,
    output_dir: PathBuf,
}

fn main() {
    let start = Instant::now();

    let runtime = Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
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

    println!("Time: {:?}", start.elapsed());
}

async fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let config = parse_args(&args)?;

    // async read
    let content = fs::read_to_string(&config.list_path)
        .await
        .map_err(|e| format!("Cannot read file: {e}"))?;

    let entries: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // async tasks
    let tasks = entries.into_iter().enumerate().map(|(idx, entry)| {
        let config = config.clone();

        tokio::spawn(async move {
            if entry.trim().is_empty() {
                return;
            }

            if let Err(e) = process_entry(entry, config, idx).await {
                eprintln!("Error #{idx}: {e}");
            }
        })
    });

    join_all(tasks).await;

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
        output_dir,
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

async fn process_entry(
    entry: String,
    config: Config,
    index: usize,
) -> Result<(), String> {
    let img = load_image_from_path(&entry)?;

    // CPU-bound → blocking pool
    let resized = tokio::task::spawn_blocking(move || {
        resize_image(img, config.width, config.height)
    })
    .await
    .map_err(|_| "CPU task failed")?;

    let output_path = build_output_path(
        &entry,
        &config.output_dir,
        config.width,
        config.height,
        index,
    );

    tokio::task::spawn_blocking(move || {
        resized.save(&output_path)
    })
    .await
    .map_err(|_| "Save task failed")?
    .map_err(|e| format!("Save error: {e}"))?;

    println!("Saved: {}", output_path.display());

    Ok(())
}

fn load_image_from_path(path: &str) -> Result<DynamicImage, String> {
    image::open(path).map_err(|e| format!("file error: {e}"))
}

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