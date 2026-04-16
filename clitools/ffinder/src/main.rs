use std::fs;
use std::io::{self, BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Использование: filefinder <путь> [--name <шаблон>] [--size <макс_размер_КБ>]");
        std::process::exit(1);
    }

    let path = args[1].clone(); // Клонируем строку, чтобы избежать проблем с временем жизни
    let name_filter = get_option(&args, "--name");
    let size_limit = get_size_limit(&args);

    let results = Arc::new(Mutex::new(Vec::new()));

    // Многопоточный обход директорий
    let handle = thread::spawn({
        let results = Arc::clone(&results);
        move || {
            if let Err(e) = search_directory(&path, &name_filter, &size_limit, &results) {
                eprintln!("Ошибка при обходе: {}", e);
            }
        }
    });

    handle.join().unwrap();

    // Вывод результатов
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    let results_lock = results.lock().unwrap();
    for file_path in &*results_lock {
        writeln!(writer, "{}", file_path)?;
    }
    writer.flush()?;

    Ok(())
}

fn get_option(args: &[String], flag: &str) -> Option<String> {
    if let Some(pos) = args.iter().position(|a| a == flag) {
        if pos + 1 < args.len() {
            return Some(args[pos + 1].clone());
        }
    }
    None
}

fn get_size_limit(args: &[String]) -> Option<u64> {
    get_option(args, "--size").and_then(|s| s.parse::<u64>().ok().map(|kb| kb * 1024))
}

fn search_directory(
    path: &str,
    name_filter: &Option<String>,
    size_limit: &Option<u64>,
    results: &Mutex<Vec<String>>,
) -> io::Result<()> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let path = entry.path();

            if file_type.is_dir() {
                search_directory(&path.to_string_lossy(), name_filter, size_limit, results)?;
            } else if file_type.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Проверка фильтра по имени
                if name_filter.as_ref().map_or(true, |f| file_name.contains(f)) {
                    // Проверка размера
                    match size_limit {
                Some(limit) => {
                    if let Ok(metadata) = fs::metadata(&path) {
                        if metadata.len() <= *limit {
                    let mut results = results.lock().unwrap();
            results.push(path.to_string_lossy().to_string());
                }
            }
                }
                None => {
            let mut results = results.lock().unwrap();
            results.push(path.to_string_lossy().to_string());
                }
            }
                }
            }
        }
    }
    Ok(())
}

