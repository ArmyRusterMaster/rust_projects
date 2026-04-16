use clap::Parser;
use image::DynamicImage;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

mod presets;
mod processor;
mod utils;

#[derive(Parser)]
#[command(name = "im-rs", version = "0.1.0", about = "Fast SIMD Image Resizer")]
struct Args {
    /// Входной файл или папка
    #[arg(short, long)]
    input: PathBuf,

    /// Папка для сохранения результата
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Ширина (умный ресайз, если не указана высота)
    #[arg(long)]
    width: Option<u32>,

    /// Высота (умный ресайз, если не указана ширина)
    #[arg(long)]
    height: Option<u32>,

    /// Пресет: insta-post, insta-story, yt
    #[arg(long)]
    preset: Option<String>,

    /// Формат: jpg, png, webp
    #[arg(short, long, default_value = "webp")]
    format: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Создаем выходную папку
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output)?;
    }

    // Собираем список файлов (поддержка популярных форматов)
    let entries: Vec<PathBuf> = if args.input.is_dir() {
        std::fs::read_dir(&args.input)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| is_image(p))
            .collect()
    } else {
        vec![args.input.clone()]
    };

    if entries.is_empty() {
        println!("Изображения не найдены.");
        return Ok(());
    }

    // Настройка прогресс-бара
    let pb = ProgressBar::new(entries.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
        .progress_chars("#>-"));

    // Параллельная обработка через Rayon
    entries.into_par_iter()
        .progress_with(pb.clone())
        .for_each(|path| {
            if let Err(e) = process_single_file(&path, &args) {
                eprintln!("Ошибка при обработке {:?}: {}", path, e);
            }
        });

    pb.finish_with_message("Готово!");
    Ok(())
}

fn process_single_file(path: &Path, args: &Args) -> Result<()> {
    // 1. Загрузка
    let img = image::open(path).context("Ошибка открытия")?;
    
    // 2. Определение размеров (Пресет или Умный ресайз)
    let (mut target_w, mut target_h) = (args.width, args.height);
    
    if let Some(preset_name) = &args.preset {
        if let Some(p) = presets::SocialPreset::from_str(preset_name) {
            let (pw, ph) = p.dimensions();
            target_w = Some(pw);
            target_h = Some(ph);
        }
    }

    let (final_w, final_h) = utils::calculate_dimensions(
        img.width(), 
        img.height(), 
        target_w, 
        target_h
    );

    // 3. Ресайз (SIMD внутри)
    let resized_buf = processor::resize_image(img, final_w, final_h)?;

    // 4. Формирование имени и сохранение
    let file_stem = path.file_stem().unwrap().to_str().unwrap();
    let out_path = args.output.join(format!("{}.{}", file_stem, args.format));
    
    DynamicImage::ImageRgba8(resized_buf).save(out_path)?;

    Ok(())
}

fn is_image(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp")
}

