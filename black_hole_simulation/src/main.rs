use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType},
    execute,
    cursor::MoveTo,
    style::Print,
};
use std::io::{Write, stdout};
use rand::Rng;

const WIDTH: u16 = 80;
const HEIGHT: u16 = 24;
const BLACK_HOLE_X: u16 = WIDTH / 2;
const BLACK_HOLE_Y: u16 = HEIGHT / 2;
const EVENT_HORIZON: f64 = 5.0; // радиус горизонта событий
const G: f64 = 100.0;     // условная гравитационная постоянная

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
}

impl Particle {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x: rng.gen_range(0.0..WIDTH as f64),
            y: rng.gen_range(0.0..HEIGHT as f64),
            vx: rng.gen_range(-0.5..0.5),
            vy: rng.gen_range(-0.5..0.5),
        }
    }

    fn update(&mut self) {
        let dx = self.x - BLACK_HOLE_X as f64;
        let dy = self.y - BLACK_HOLE_Y as f64;
        let r_squared = dx * dx + dy * dy;
        let r = r_squared.sqrt();

        if r < EVENT_HORIZON {
            // Частица поглощена — сбрасываем её
            *self = Particle::new();
            return;
        }

        // Расчёт гравитационного ускорения
        let a = G / r_squared;
        let ax = -a * dx / r;
        let ay = -a * dy / r;

        // Обновление скорости и позиции
        self.vx += ax;
        self.vy += ay;
        self.x += self.vx;
        self.y += self.vy;

        // Ограничение движения в пределах экрана
        if self.x < 0.0 || self.x >= WIDTH as f64 {
            self.vx = -self.vx;
        }
        if self.y < 0.0 || self.y >= HEIGHT as f64 {
            self.vy = -self.vy;
        }
    }
}

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();

    // Инициализация частиц
    let mut particles: Vec<Particle> = (0..50).map(|_| Particle::new()).collect();

    loop {
        // Очистка экрана
        execute!(stdout, Clear(ClearType::All))?;

        // Отрисовка чёрной дыры
        execute!(
            stdout,
            MoveTo(BLACK_HOLE_X, BLACK_HOLE_Y),
            Print("●")
        )?;

        // Обновление и отрисовка частиц
        for particle in particles.iter_mut() {
            particle.update();

            let x = particle.x as u16;
            let y = particle.y as u16;

            if x < WIDTH && y < HEIGHT {
                execute!(stdout, MoveTo(x, y), Print("·"))?;
            }
        }

        stdout.flush()?;
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    disable_raw_mode()?;
    Ok(())
}

