use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    println!("Подсчёт строк кода в папке с фильтрацией по расширениям");
    println!("Введите путь к папке:");

    let mut path_input = String::new();
    io::stdin().read_line(&mut path_input)?;
    let folder_path = path_input.trim();

    if !Path::new(folder_path).is_dir() {
        eprintln!("Ошибка: указанная папка не существует или не является директорией");
        return Ok(());
    }

    println!("Введите расширения файлов для подсчёта (через пробел, например: rs txt md):");
    let mut ext_input = String::new();
    io::stdin().read_line(&mut ext_input)?;
    let extensions: Vec<String> = ext_input
        .trim()
        .split_whitespace()
        .map(String::from)
        .collect();

    if extensions.is_empty() {
        println!("Не указаны расширения для фильтрации. Будут обработаны все файлы.");
    }

    let result = count_lines_in_folder(folder_path, &extensions)?;

    println!("\n--- Результаты подсчёта ---");
    println!("Всего файлов обработано: {}", result.total_files);
    println!("Всего строк кода: {}", result.total_lines);
    println!("Расширения для поиска: {}", extensions.join(", "));
    println!("Файлы с кодом:");

    for (file_path, line_count) in result.file_stats {
        println!("  {}: {} строк", file_path, line_count);
    }

    Ok(())
}

struct CountResult {
    total_files: usize,
    total_lines: usize,
    file_stats: Vec<(String, usize)>,
}

fn count_lines_in_folder(folder_path: &str, extensions: &[String]) -> io::Result<CountResult> {
    let mut total_files = 0;
    let mut total_lines = 0;
    let mut file_stats = Vec::new();

    // Перебираем все файлы в папке
    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();

        // Пропускаем директории
        if path.is_dir() {
            continue;
        }

        // Фильтруем по расширениям, если они указаны
        if !extensions.is_empty() && !should_process_file(&path, extensions) {
            continue;
        }

        match count_lines_in_file(&path) {
            Ok(line_count) => {
                total_files += 1;
                total_lines += line_count;
                file_stats.push((path.display().to_string(), line_count));
            }
            Err(e) => {
                eprintln!("Ошибка чтения файла {}: {}", path.display(), e);
            }
        }
    }

    Ok(CountResult {
        total_files,
        total_lines,
        file_stats,
    })
}

/// Проверяет, соответствует ли файл одному из указанных расширений
fn should_process_file(path: &Path, extensions: &[String]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.iter().any(|e| e == ext))
        .unwrap_or(false)
}

/// Считает количество строк в файле
fn count_lines_in_file(file_path: &Path) -> io::Result<usize> {
    let content = fs::read_to_string(file_path)?;
    Ok(content.lines().count())
}
