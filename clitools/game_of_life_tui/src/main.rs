use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{canvas::{Canvas, Points}, Block, Borders, Paragraph},
    Terminal,
};
use std::{io, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};

struct SimpleRng { state: u64 }
impl SimpleRng {
    fn new() -> Self {
        let s = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        Self { state: s as u64 }
    }
    fn gen_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as f64 / u32::MAX as f64
    }
}

struct App {
    grid: Vec<Vec<u8>>,
    rows: usize,
    cols: usize,
}

impl App {
    fn new(w: u16, h: u16) -> Self {
        // Умножаем на 2 и 4, так как один символ Брайля покрывает 2x4 точки
        let cols = (w.saturating_sub(2) * 2) as usize;
        let rows = (h.saturating_sub(5) * 4) as usize;
        let mut rng = SimpleRng::new();
        let grid = (0..rows)
            .map(|_| (0..cols).map(|_| if rng.gen_f64() < 0.15 { 1 } else { 0 }).collect())
            .collect();
        Self { grid, rows, cols }
    }

    fn update(&mut self) {
        let mut next_grid = vec![vec![0; self.cols]; self.rows];
        for r in 0..self.rows {
            for c in 0..self.cols {
                let mut neighbors = 0;
                for i in -1..=1 {
                    for j in -1..=1 {
                        if i == 0 && j == 0 { continue; }
                        let nr = (r as i32 + i).rem_euclid(self.rows as i32) as usize;
                        let nc = (c as i32 + j).rem_euclid(self.cols as i32) as usize;
                        if self.grid[nr][nc] > 0 { neighbors += 1; }
                    }
                }
                let age = self.grid[r][c];
                if age > 0 {
                    if neighbors == 2 || neighbors == 3 {
                        next_grid[r][c] = age.saturating_add(1);
                    }
                } else if neighbors == 3 {
                    next_grid[r][c] = 1;
                }
            }
        }
        self.grid = next_grid;
    }

    fn count_alive(&self) -> usize {
        self.grid.iter().flat_map(|r| r.iter()).filter(|&&c| c > 0).count()
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new(terminal.size()?.width, terminal.size()?.height);
    let mut last_tick = Instant::now();
    let mut paused = false;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(f.area());

            let alive_count = app.count_alive();

            // 1. Поле игры с использованием шрифта Брайля
            let canvas = Canvas::default()
                .block(Block::default().borders(Borders::ALL).title(" LIFE SIMULATOR (BRAILLE) "))
                .x_bounds([0.0, app.cols as f64])
                .y_bounds([0.0, app.rows as f64])
                .paint(|ctx| {
                    let mut points = Vec::new();
                    for r in 0..app.rows {
                        for c in 0..app.cols {
                            if app.grid[r][c] > 0 {
                                // Canvas рисует снизу вверх, поэтому инвертируем Y
                                points.push((c as f64, (app.rows as f64 - 1.0 - r as f64)));
                            }
                        }
                    }
                    // draw(&Points) автоматически использует Брайль в Ratatui
                    ctx.draw(&Points {
                        coords: &points,
                        color: Color::Cyan,
                    });
                });
            f.render_widget(canvas, chunks[0]);

            // 2. Инструкция + Счетчик
            let status = if paused { "ПАУЗА".red().bold() } else { "ИДЕТ".green() };
            let help_msg = format!(
                "Статус: {} | Живых: {} | Пробел: Пауза | R: Ресет | Q: Выход",
                status, alive_count
            );

            let help = Paragraph::new(help_msg)
                .block(Block::default().borders(Borders::ALL).title(" Управление и Инфо "))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(help, chunks[1]);
        })?;

        let timeout = Duration::from_millis(50).checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('r') => app = App::new(terminal.size()?.width, terminal.size()?.height),
                    KeyCode::Char(' ') => paused = !paused,
                    _ => {}
                }
            }
        }

        if !paused && last_tick.elapsed() >= Duration::from_millis(50) {
            app.update();
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

