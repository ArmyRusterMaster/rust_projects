use crossterm::{
    cursor,
    event::{self, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{stdout, Write};
use std::time::Duration;
use std::f32::consts::PI;

const MAP_W: usize = 32;
const MAP_H: usize = 32;

fn generate_map() -> Vec<u8> {
    let mut map = vec![b'#'; MAP_W * MAP_H];
    let mut x = MAP_W / 2;
    let mut y = MAP_H / 2;
    let mut rng = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    for _ in 0..600 {
        map[y * MAP_W + x] = b'.';
        rng = (rng * 1103515245 + 12345) & 0x7fffffff;
        match rng % 4 {
            0 => if x < MAP_W - 2 { x += 1 },
            1 => if x > 1 { x -= 1 },
            2 => if y < MAP_H - 2 { y += 1 },
            _ => if y > 1 { y -= 1 },
        }
    }
    map
}

fn main() {
    let map = generate_map();
    let mut p_x = MAP_W as f32 / 2.0;
    let mut p_y = MAP_H as f32 / 2.0;
    let mut p_a = 0.0_f32;
    let fov = PI / 3.0;

    let mut stdout = stdout();
    terminal::enable_raw_mode().unwrap();
    stdout.execute(EnterAlternateScreen).unwrap();

    loop {
        let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
        let screen_w = term_w as usize;
        let screen_h = term_h as usize;

        if event::poll(Duration::from_millis(16)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                let (old_x, old_y) = (p_x, p_y);
                match key.code {
                    KeyCode::Char('w') => { p_x += p_a.sin() * 0.4; p_y += p_a.cos() * 0.4; }
                    KeyCode::Char('s') => { p_x -= p_a.sin() * 0.4; p_y -= p_a.cos() * 0.4; }
                    KeyCode::Char('a') => p_a -= 0.2,
                    KeyCode::Char('d') => p_a += 0.2,
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    _ => {}
                }
                if map[(p_y as usize).clamp(0, MAP_H-1) * MAP_W + (p_x as usize).clamp(0, MAP_W-1)] == b'#' {
                    p_x = old_x; p_y = old_y;
                }
            }
        }

        let mut buffer = vec![vec![' '; screen_w]; screen_h];

        // 1. РЕНДЕРИНГ 3D
        for x in 0..screen_w {
            let ray_angle = (p_a - fov / 2.0) + (x as f32 / screen_w as f32) * fov;
            let mut dist = 0.0;
            let (eye_x, eye_y) = (ray_angle.sin(), ray_angle.cos());

            while dist < 16.0 {
                dist += 0.1;
                let tx = (p_x + eye_x * dist) as usize;
                let ty = (p_y + eye_y * dist) as usize;
                if tx >= MAP_W || ty >= MAP_H || map[ty * MAP_W + tx] == b'#' { break; }
            }

            let corrected_dist = dist * (ray_angle - p_a).cos();
            let wall_h = (screen_h as f32 / (corrected_dist + 0.1)) * 0.5;
            let ceil = (screen_h as f32 / 2.0 - wall_h).max(0.0) as usize;
            let floor = (screen_h as f32 / 2.0 + wall_h).min(screen_h as f32 - 1.0) as usize;

            for y in 0..screen_h {
                if y < ceil {
                    buffer[y][x] = ' ';
                } else if y >= ceil && y <= floor {
                    // ТЕНИ: Используем реальную дистанцию 'dist', а не исправленную
                    buffer[y][x] = if dist < 2.5 { '█' } 
                                   else if dist < 5.0 { '▓' } 
                                   else if dist < 8.0 { '▒' } 
                                   else if dist < 12.0 { '░' }
                                   else { ' ' }; 
                } else {
                    // Пол тоже затеняем
                    buffer[y][x] = if y as f32 > screen_h as f32 * 0.8 { '=' } else { '.' };
                }
            }
        }

        // 2. МИНИКАРТА (с коррекцией пропорций 2:1)
        let mm_scale = 1; // Коэффициент размера
        for my in 0..MAP_H {
            for mx in 0..MAP_W {
                let sy = my / 2; // Уменьшаем в 2 раза по вертикали
                let sx = mx;     // По горизонтали оставляем (так как символы узкие)
                
                if sy + 1 < screen_h && sx * 2 + 2 < screen_w {
                    let char = if mx == p_x as usize && my == p_y as usize {
                        '@'
                    } else if map[my * MAP_W + mx] == b'#' {
                        '#'
                    } else {
                        ' '
                    };
                    
                    if char != ' ' {
                        buffer[sy + 1][sx + 1] = char;
                    }
                }
            }
        }

        // 3. ВЫВОД
        let mut frame = String::with_capacity(screen_w * screen_h);
        for row in buffer {
            frame.extend(row);
        }

        stdout.execute(cursor::MoveTo(0, 0)).unwrap();
        write!(stdout, "{}", frame).unwrap();
        stdout.flush().unwrap();
    }

    stdout.execute(LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();
}

