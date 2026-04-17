use clap::Parser;
// Явно импортируем трейт для работы прогресс-бара с Rayon
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, io, path::PathBuf, sync::Mutex};
use walkdir::{WalkDir, DirEntry};

#[derive(Parser)]
#[command(name = "FileDedup", version = "1.0", about = "🔥 Быстрый поиск дубликатов")]
struct Args {
    #[arg(short, long, help = "Расширение (png, mp3)")]
    ext: Option<String>,

    #[arg(default_value = ".", help = "Путь")]
    path: String,

    #[arg(short, long, default_value_t = 4, help = "Потоки")]
    threads: usize,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .unwrap();

    println!("📂 Сканирование: {}", args.path);
    
    let entries: Vec<DirEntry> = WalkDir::new(&args.path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| {
            if let Some(ref filter_ext) = args.ext {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.eq_ignore_ascii_case(filter_ext))
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .collect();

    if entries.is_empty() {
        println!("❌ Файлы не найдены.");
        return Ok(());
    }

    let pb = ProgressBar::new(entries.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap());

    let results: Mutex<HashMap<String, Vec<PathBuf>>> = Mutex::new(HashMap::new());

    // Добавили аннотацию типа |entry: DirEntry|
    entries.into_par_iter().progress_with(pb).for_each(|entry: DirEntry| {
        let path = entry.path();
        if let Ok(hash) = hash_file(path) {
            let mut map = results.lock().unwrap();
            map.entry(hash).or_insert(vec![]).push(path.to_path_buf());
        }
    });

    print_results(results.into_inner().unwrap());
    Ok(())
}

fn hash_file(path: &std::path::Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(hex::encode(hasher.finalize()))
}

fn print_results(map: HashMap<String, Vec<PathBuf>>) {
    let dups: Vec<_> = map.into_iter().filter(|(_, v)| v.len() > 1).collect();
    
    if dups.is_empty() {
        println!("\n✅ Дубликатов не найдено.");
    } else {
        println!("\n🚀 Найдено {} групп:", dups.len());
        for (hash, paths) in dups {
            println!("\nSHA256: {}...", &hash[..16]);
            for p in paths {
                println!("  [DUP] {:?}", p);
            }
        }
    }
}

