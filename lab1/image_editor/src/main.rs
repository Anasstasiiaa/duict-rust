use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::DynamicImage;
//cargo run -- --files images.txt --resize 800x600
/// Configuration parsed from CLI arguments and environment.
struct Config {
    list_path: PathBuf,
    width: u32,
    height: u32,
    output_dir: PathBuf,
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

    let file = File::open(&config.list_path)
        .map_err(|e| format!("Не вдається відкрити файл зі списком: {e}"))?;
    let reader = BufReader::new(file);

    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Помилка читання рядка {idx}: {e}"))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Err(err) = process_entry(trimmed, &config, idx) {
            eprintln!("Помилка для запису #{idx} (`{trimmed}`): {err}");
        }
    }

    Ok(())
}

fn parse_args(args: &[String]) -> Result<Config, String> {
    // args[0] is the program name.
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

    let list_path =
        list_path.ok_or_else(|| "Не вказано файл зі списком (прапор --files)".to_string())?;
    let (width, height) =
        resize.ok_or_else(|| "Не вказано розмір (прапор --resize widthxheight)".to_string())?;

    let output_dir_str = env::var("MYME_FILES_PATH")
        .map_err(|_| "Змінна середовища MYME_FILES_PATH не встановлена".to_string())?;
    let output_dir = PathBuf::from(output_dir_str);

    if !output_dir.exists() {
        return Err(format!(
            "Каталог, вказаний у MYME_FILES_PATH, не існує: {}",
            output_dir.display()
        ));
    }

    Ok(Config {
        list_path,
        width,
        height,
        output_dir,
    })
}

fn parse_resize(s: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
        return Err("Невірний формат для --resize, очікується widthxheight".into());
    }
    let width: u32 = parts[0]
        .parse()
        .map_err(|_| "Ширина повинна бути додатнім числом".to_string())?;
    let height: u32 = parts[1]
        .parse()
        .map_err(|_| "Висота повинна бути додатнім числом".to_string())?;
    if width == 0 || height == 0 {
        return Err("Ширина та висота повинні бути більше нуля".into());
    }
    Ok((width, height))
}

fn process_entry(entry: &str, config: &Config, index: usize) -> Result<(), String> {
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

    resized.save(&output_path).map_err(|e| {
        format!(
            "Не вдалося зберегти зображення в {}: {e}",
            output_path.display()
        )
    })?;

    println!("Збережено: {}", output_path.display());
    Ok(())
}

fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

fn load_image_from_url(url: &str) -> Result<DynamicImage, String> {
    let response = reqwest::blocking::get(url).map_err(|e| format!("Помилка HTTP-запиту: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Сервер повернув статус: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .map_err(|e| format!("Не вдалося прочитати тіло відповіді: {e}"))?;

    image::load_from_memory(&bytes)
        .map_err(|e| format!("Не вдалося завантажити зображення з памʼяті: {e}"))
}

fn load_image_from_path(path: &str) -> Result<DynamicImage, String> {
    let path_obj = Path::new(path);

    image::open(path_obj).map_err(|e| {
        format!(
            "Не вдалося відкрити зображення за шляхом {}: {e}",
            path_obj.display()
        )
    })
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
    let base_name = if is_url(original) {
        // спробувати взяти імʼя файлу з URL
        original
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("image")
    } else {
        Path::new(original)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image")
    };

    // Забрати розширення, якщо є
    let base_without_ext = base_name
        .rsplit_once('.')
        .map(|(name, _)| name)
        .unwrap_or(base_name);

    let file_name = format!("{base_without_ext}_{width}x{height}_{index}.png");
    output_dir.join(file_name)
}
