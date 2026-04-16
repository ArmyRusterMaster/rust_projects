use anyhow::Result;
use crossterm::{
    event::{self, Event, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, prelude::*};
use std::{io, path::PathBuf, time::{Duration, Instant}};

// Импортируем модули из нашей библиотеки
use hex_editor::{app::App, handler, ui};

fn main() -> Result<()> {
    // 1. Получаем путь к файлу из аргументов CLI
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Ошибка: Укажите путь к файлу.");
        eprintln!("Пример: cargo run -- my_file.bin");
        std::process::exit(1);
    }
    let path = PathBuf::from(&args[1]);

    // 2. Инициализация состояния приложения
    let mut app = App::new(path.clone())?;

    // 3. Настройка терминала (Raw mode + Alternate Screen)
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 4. Основной цикл приложения
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(16); // ~60 FPS

    loop {
        // Отрисовка кадра
        terminal.draw(|f| ui::render(f, &mut app))?;

        // Обработка событий с таймаутом
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if handler::handle_key_events(key, &mut app, &path) {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.adjust_scroll(); // Обновляем скролл согласно позиции курсора
            last_tick = Instant::now();
        }
    }

    // 5. Восстановление терминала перед выходом
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

