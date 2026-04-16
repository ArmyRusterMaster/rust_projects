use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> io::Result<()> {
    println!("Параллельный подсчёт строк кода во всём дереве папок с фильтрацией по расширениям");
    println!("Введите путь к корневой папке:");

    let mut path_input = String::new();
    io::stdin().read_line(&mut path_input)?;
    let root_path = path_input.trim();

    if !Path::new(root_path).is_dir() {
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

    // Собираем все файлы для обработки
    let files_to_process = collect_files_recursively(root_path, &extensions)?;

    // Используем Arc<Mutex<>> для безопасного совместного доступа между потоками
    let total_files = Arc::new(Mutex::new(0));
    let total_lines = Arc::new(Mutex::new(0));
    let file_stats = Arc::new(Mutex::new(Vec::new()));

    // Создаём потоки для параллельной обработки
    let mut handles = vec![];

    for file_path in files_to_process {
        let total_files_clone = Arc::clone(&total_files);
        let total_lines_clone = Arc::clone(&total_lines);
        let file_stats_clone = Arc::clone(&file_stats);

        let handle = thread::spawn(move || {
            match count_lines_in_file(&file_path) {
                Ok(line_count) => {
                    // Обновляем общие счётчики
                    *total_files_clone.lock().unwrap() += 1;
                    *total_lines_clone.lock().unwrap() += line_count;

                    // Добавляем статистику по файлу
                    file_stats_clone.lock().unwrap().push((
                file_path.display().to_string(),
                line_count
            ));
                }
                Err(e) => {
                    eprintln!("Ошибка чтения файла {}: {}", file_path.display(), e);
                }
            }
        });

        handles.push(handle);
    }

    // Ждём завершения всех потоков
    for handle in handles {
        handle.join().unwrap();
    }

    // Получаем итоговые результаты
    let total_files_final = *total_files.lock().unwrap();
    let total_lines_final = *total_lines.lock().unwrap();
    let mut file_stats_final = file_stats.lock().unwrap().clone();

    // Сортируем файлы по количеству строк (по убыванию)
    file_stats_final.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n--- Результаты подсчёта ---");
    println!("Всего файлов обработано: {}", total_files_final);
    println!("Всего строк кода: {}", total_lines_final);
    println!("Расширения для поиска: {}", extensions.join(", "));
    println!("Файлы с кодом (отсортировано по убыванию строк):");

    for (file_path, line_count) in file_stats_final {
        println!("  {}: {} строк", file_path, line_count);
    }

    Ok(())
}

/// Рекурсивно собирает все файлы в дереве папок, фильтруя по расширениям
fn collect_files_recursively(folder_path: &str, extensions: &[String]) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Рекурсивный вызов для подпапок
            let mut subfolder_files = collect_files_recursively(&path.to_string_lossy(), extensions)?;
            files.append(&mut subfolder_files);
        } else {
            // Фильтруем по расширениям, если они указаны
            if extensions.is_empty() || should_process_file(&path, extensions) {
                files.push(path);
            }
        }
    }

    Ok(files)
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

