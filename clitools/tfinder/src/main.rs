use std::fs;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Использование: textfinder <фрагмент> [--context <строк>] [--output <файл>]");
        std::process::exit(1);
    }

    let pattern = args[1].clone(); // Клонируем, чтобы избежать проблем с временем жизни
    let context = get_context(&args);
    let output_file = get_output_file(&args);

    // Чтение путей из стандартного ввода
    let stdin = io::stdin();
    let lines = BufReader::new(stdin.lock()).lines();
    let file_paths: Vec<String> = lines.filter_map(Result::ok).collect();

    let matches = Arc::new(Mutex::new(Vec::new()));
    let chunk_size = 4; // Обрабатываем по 4 файла за раз

    let handles: Vec<_> = file_paths.chunks(chunk_size).map(|chunk| {
        let matches = Arc::clone(&matches);
        let chunk = chunk.to_vec();
        let pattern = pattern.clone(); // Клонируем pattern для каждого потока

        thread::spawn(move || {
            for path in chunk {
                if let Err(e) = process_file(&path, &pattern, context, &matches) {
                    eprintln!("Ошибка чтения {}: {}", path, e);
                }
            }
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Вывод результатов
    let matches_lock = matches.lock().unwrap();
    match output_file {
        Some(filename) => {
            let mut file = fs::File::create(filename)?;
            for m in &*matches_lock {
                writeln!(file, "{}", m)?;
            }
        }
        None => {
            let stdout = io::stdout();
            let mut writer = BufWriter::new(stdout.lock());
            for m in &*matches_lock {
                writeln!(writer, "{}", m)?;
            }
            writer.flush()?;
        }
    }

    Ok(())
}

fn get_context(args: &[String]) -> usize {
    get_option(args, "--context").and_then(|s| s.parse().ok()).unwrap_or(0)
}

fn get_output_file(args: &[String]) -> Option<String> {
    get_option(args, "--output")
}

fn get_option(args: &[String], flag: &str) -> Option<String> {
    if let Some(pos) = args.iter().position(|a| a == flag) {
        if pos + 1 < args.len() {
            return Some(args[pos + 1].clone());
        }
    }
    None
}

fn process_file(
    path: &str,
    pattern: &str,
    context: usize,
    matches: &Mutex<Vec<String>>,
) -> io::Result<()> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.contains(pattern) {
            let start = if i >= context { i - context } else { 0 };
            let end = (i + context + 1).min(lines.len());
            let context_lines: Vec<&str> = lines[start..end].to_vec();
            let result = format!("{}:{}\n{}", path, i + 1, context_lines.join("\n"));
            let mut matches = matches.lock().unwrap();
            matches.push(result);
        }
    }
    Ok(())
}

