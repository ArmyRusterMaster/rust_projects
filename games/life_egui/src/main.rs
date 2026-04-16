use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Duration;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 450;
const CELL_SIZE: u32 = 10;
const GRID_W: usize = (WIDTH / CELL_SIZE) as usize;
const GRID_H: usize = ((HEIGHT - 50) / CELL_SIZE) as usize; // Оставляем место под панель

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("Game of Life - Termux SDL2", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software() // <--- Добавь это
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut grid = vec![vec![false; GRID_W]; GRID_H];
    let mut running = false;

    // Рандомное заполнение в начале
    let mut rng = rand::thread_rng();
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            grid[y][x] = rng.gen_bool(0.2);
        }
    }

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown { x, y, .. } => {
                    if y > (HEIGHT - 50) as i32 {
                        // Логика кнопок (упрощенно: левая половина - старт/стоп, правая - очистка)
                        if x < (WIDTH / 2) as i32 {
                            running = !running;
                        } else {
                            grid = vec![vec![false; GRID_W]; GRID_H];
                        }
                    } else {
                        let gx = (x as u32 / CELL_SIZE) as usize;
                        let gy = (y as u32 / CELL_SIZE) as usize;
                        if gy < GRID_H && gx < GRID_W {
                            grid[gy][gx] = !grid[gy][gx];
                        }
                    }
                }
                _ => {}
            }
        }

        if running {
            let mut next_grid = grid.clone();
            for y in 0..GRID_H {
                for x in 0..GRID_W {
                    let mut n = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let ny = (y as i32 + dy).rem_euclid(GRID_H as i32) as usize;
                            let nx = (x as i32 + dx).rem_euclid(GRID_W as i32) as usize;
                            if grid[ny][nx] {
                                n += 1;
                            }
                        }
                    }
                    next_grid[y][x] = match (grid[y][x], n) {
                        (true, 2) | (true, 3) => true,
                        (false, 3) => true,
                        _ => false,
                    };
                }
            }
            grid = next_grid;
        }

        canvas.set_draw_color(Color::RGB(20, 20, 20));
        canvas.clear();

        // Рисуем сетку
        canvas.set_draw_color(Color::RGB(0, 255, 0));
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                if grid[y][x] {
                    canvas.fill_rect(Rect::new(
                        (x as u32 * CELL_SIZE) as i32,
                        (y as u32 * CELL_SIZE) as i32,
                        CELL_SIZE - 1,
                        CELL_SIZE - 1,
                    ))?;
                }
            }
        }

        // Панель управления (снизу)
        canvas.set_draw_color(Color::RGB(50, 50, 50));
        canvas.fill_rect(Rect::new(0, (HEIGHT - 50) as i32, WIDTH, 50))?;

        // Рисуем "Кнопки" (текст в SDL2 сложнее, сделаем цветные прямоугольники)
        canvas.set_draw_color(if running {
            Color::RGB(255, 0, 0)
        } else {
            Color::RGB(0, 200, 0)
        });
        canvas.fill_rect(Rect::new(10, (HEIGHT - 40) as i32, 100, 30))?; // Start/Stop

        canvas.set_draw_color(Color::RGB(100, 100, 255));
        canvas.fill_rect(Rect::new(WIDTH as i32 - 110, (HEIGHT - 40) as i32, 100, 30))?; // Clear

        canvas.present();
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}
