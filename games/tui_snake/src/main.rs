use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::canvas::*, widgets::*};
use std::{io, time::{Duration, Instant}};

#[derive(PartialEq, Clone, Copy)]
struct Point { x: u16, y: u16 }

struct Game {
    snake: Vec<Point>,
    food: Point,
    dir: KeyCode,
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut game = Game {
        snake: vec![Point { x: 10, y: 10 }],
        food: Point { x: 5, y: 5 },
        dir: KeyCode::Right,
    };

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(150);

    loop {
        terminal.draw(|f| {
            let canvas = Canvas::default()
                .block(Block::default().borders(Borders::ALL).title(" Snake Rust (Esc to exit) "))
                .x_bounds([0.0, 40.0])
                .y_bounds([0.0, 20.0])
                .paint(|ctx: &mut Context| {
                    ctx.print(game.food.x as f64, game.food.y as f64, "🍎");
                    for p in &game.snake {
                        ctx.print(p.x as f64, p.y as f64, "🟩");
                    }
                });
            f.render_widget(canvas, f.area());
        })?;

        // Исправленный расчет ожидания (timeout)
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => game.dir = key.code,
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            let mut head = game.snake[0];
            match game.dir {
                KeyCode::Up => head.y = head.y.saturating_add(1),
                KeyCode::Down => head.y = head.y.saturating_sub(1),
                KeyCode::Left => head.x = head.x.saturating_sub(1),
                KeyCode::Right => head.x = head.x.saturating_add(1),
                _ => {}
            }
            
            game.snake.insert(0, head);
            if head == game.food {
                // Новая позиция еды (простая логика)
                game.food = Point { 
                    x: (head.x + 7) % 38 + 1, 
                    y: (head.y + 3) % 18 + 1 
                };
            } else {
                game.snake.pop();
            }
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

