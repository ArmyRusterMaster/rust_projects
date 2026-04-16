use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::time::Duration;

enum FractalType { Mandelbrot, Julia, BurningShip }

struct App {
    x_center: f64,
    y_center: f64,
    zoom: f64,
    fractal_type: FractalType,
    max_iter: usize,
}

fn main() -> Result<(), io::Error> {
    // Подготовка терминала
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App {
        x_center: -0.5,
        y_center: 0.0,
        zoom: 1.0,
        fractal_type: FractalType::Mandelbrot,
        max_iter: 40,
    };

    loop {
        terminal.draw(|f| {
            // Разделение экрана на фрактал и инфо-панель
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
                .split(f.area());

            let area = chunks[0];
            let mut lines = Vec::with_capacity(area.height as usize);

            // Отрисовка фрактала построчно
            for y in 0..area.height {
                let mut spans = Vec::with_capacity(area.width as usize);
                for x in 0..area.width {
                    // Математика координат с учетом зума и пропорций терминала
                    let nx = (x as f64 / area.width as f64 - 0.5) * (4.0 / app.zoom) + app.x_center;
                    let ny = (y as f64 / area.height as f64 - 0.5) * (4.0 / app.zoom) * 2.0 + app.y_center;

                    let iters = match app.fractal_type {
                        FractalType::Mandelbrot => get_iters(nx, ny, 0.0, 0.0, app.max_iter, false),
                        FractalType::Julia => get_iters(-0.8, 0.156, nx, ny, app.max_iter, false),
                        FractalType::BurningShip => get_iters(nx, ny, 0.0, 0.0, app.max_iter, true),
                    };
                    spans.push(get_styled_char(iters, app.max_iter));
                }
                lines.push(Line::from(spans));
            }
            f.render_widget(Paragraph::new(lines), area);

            // Нижняя панель управления
            let type_name = match app.fractal_type {
                FractalType::Mandelbrot => "Mandelbrot",
                FractalType::Julia => "Julia (-0.8, 0.156)",
                FractalType::BurningShip => "Burning Ship",
            };
            let info_text = format!(
                " Mode[1,2,3]: {} | Zoom: {:.1e} | Depth: {} | [+/-]: Zoom | Arrows: Move | Q: Exit ",
                type_name, app.zoom, app.max_iter
            );
            f.render_widget(
                Paragraph::new(info_text).block(Block::default().borders(Borders::ALL).title("Fractal TUI")),
                chunks[1]
            );
        })?;

        // Обработка ввода
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                let step = 0.15 / app.zoom;
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('й') => break,
                    // Автоматический зум с пересчетом глубины
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        app.zoom *= 1.3;
                        app.max_iter = (40.0 + app.zoom.log10() * 25.0) as usize;
                    },
                    KeyCode::Char('-') => {
                        app.zoom /= 1.3;
                        app.max_iter = (40.0 + app.zoom.log10() * 25.0).max(40.0) as usize;
                    },
                    // Перемещение
                    KeyCode::Up => app.y_center -= step,
                    KeyCode::Down => app.y_center += step,
                    KeyCode::Left => app.x_center -= step,
                    KeyCode::Right => app.x_center += step,
                    // Переключение типов
                    KeyCode::Char('1') => app.fractal_type = FractalType::Mandelbrot,
                    KeyCode::Char('2') => app.fractal_type = FractalType::Julia,
                    KeyCode::Char('3') => app.fractal_type = FractalType::BurningShip,
                    // Ручная доп-глубина
                    KeyCode::Char(']') | KeyCode::Char('ъ') => app.max_iter += 25,
                    KeyCode::Char('[') | KeyCode::Char('х') => {
                        if app.max_iter > 25 { app.max_iter -= 25; }
                    }
                    _ => {}
                }
            }
        }
    }

    // Восстановление терминала
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn get_iters(cx: f64, cy: f64, mut x: f64, mut y: f64, max: usize, is_ship: bool) -> usize {
    let mut i = 0;
    while x * x + y * y <= 4.0 && i < max {
        if is_ship { x = x.abs(); y = y.abs(); }
        let xt = x * x - y * y + cx;
        y = 2.0 * x * y + cy;
        x = xt;
        i += 1;
    }
    i
}

fn get_styled_char<'a>(iters: usize, max: usize) -> Span<'a> {
    if iters == max {
        return Span::styled(" ", Style::default().bg(Color::Black));
    }

    // Богатая палитра ANSI 256-цветов
    let color_idx = (iters % 255) as u8;
    let symbol = match iters % 10 {
        0 => ".", 1 => "·", 2 => ":", 3 => "-", 
        4 => "=", 5 => "+", 6 => "*", 7 => "#", 
        8 => "%", _ => "@",
    };

    Span::styled(
        symbol,
        Style::default()
            .fg(Color::Indexed(color_idx))
            .add_modifier(Modifier::BOLD)
    )
}

