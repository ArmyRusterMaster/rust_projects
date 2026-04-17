use clap::Parser;
use dashmap::DashMap;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use mimalloc::MiMalloc;
use rayon::prelude::*;
use std::{
    fs, 
    io::{self, Read}, 
    path::PathBuf, 
    hash::Hasher,
    ffi::OsStr
};
use twox_hash::XxHash64;
use walkdir::WalkDir;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser)]
struct Args {
    #[arg(default_value = ".")]
    path: String,
    #[arg(short, long)]
    ext: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    println!("📂 Индексация...");
    let size_map: DashMap<u64, Vec<PathBuf>> = DashMap::new();
    
    // Используем par_iter даже для сбора путей, если файлов ОЧЕНЬ много
    let entries: Vec<_> = WalkDir::new(&args.path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    entries.into_par_iter().for_each(|e| {
        if let Ok(meta) = e.metadata() {
            let path = e.path().to_path_buf();
            if let Some(ref filter) = args.ext {
                if path.extension().and_then(OsStr::to_str) != Some(filter) { return; }
            }
            size_map.entry(meta.len()).or_default().push(path);
        }
    });

    let candidates: Vec<PathBuf> = size_map.into_iter()
        .filter(|(_, p)| p.len() > 1)
        .flat_map(|(_, p)| p)
        .collect();

    if candidates.is_empty() {
        println!("✅ Дубликатов не найдено.");
        return Ok(());
    }

    println!("⚡ Хеширование {} файлов...", candidates.len());
    let pb = ProgressBar::new(candidates.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}")
        .unwrap());

    let hash_map: DashMap<u64, Vec<PathBuf>> = DashMap::new();

    candidates.into_par_iter().progress_with(pb).for_each(|path| {
        // Ошибка была здесь: буфер теперь в куче (heap), а не на стеке
        if let Ok(h) = compute_fast_hash_heap(&path) {
            hash_map.entry(h).or_default().push(path);
        }
    });

    print_results(hash_map);
    Ok(())
}

fn compute_fast_hash_heap(path: &PathBuf) -> io::Result<u64> {
    let mut file = fs::File::open(path)?;
    let mut hasher = XxHash64::with_seed(0);
    
    // БУФЕР В КУЧЕ (Heap allocation)
    // 32 KB достаточно для эффективного чтения в Termux
    let mut buffer = vec![0u8; 32768]; 

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.write(&buffer[..n]);
    }
    Ok(hasher.finish())
}

fn print_results(map: DashMap<u64, Vec<PathBuf>>) {
    for res in map.iter().filter(|r| r.value().len() > 1) {
        println!("\n🎯 Hash: {:X}", res.key());
        for path in res.value() {
            println!("  📄 {:?}", path);
        }
    }
}

