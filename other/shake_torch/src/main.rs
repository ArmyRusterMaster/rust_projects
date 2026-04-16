use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use serde::Deserialize;
use std::time::{Duration, Instant};

#[derive(Deserialize, Debug)]
struct SensorData {
    #[serde(rename = "accel")]
    values: Option<[f32; 3]>,
}

fn main() {
    let mut torch_on = false;
    let threshold = 20.0; 
    let mut last_shake = Instant::now();

    println!("--- [INIT] Запуск мониторинга ---");

    // Запускаем процесс и проверяем, не закрылся ли он сразу
    let mut child = Command::new("termux-sensor")
        .args(&["-s", "accel", "-delay", "100"]) 
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // Захватываем ошибки системы
        .spawn()
        .expect("КРИТИЧЕСКАЯ ОШИБКА: Не удалось запустить termux-sensor. Проверь 'pkg install termux-api'");

    let stdout = child.stdout.take().expect("Не удалось захватить stdout");
    let reader = BufReader::new(stdout);

    println!("[LOG] Ожидание данных от акселерометра...");

    for line in reader.lines() {
        match line {
            Ok(l) => {
                // Пытаемся распарсить строку. Если ошибка — просто идем к следующей, не вылетая.
                if let Ok(data) = serde_json::from_str::<SensorData>(&l) {
                    if let Some([x, y, z]) = data.values {
                        let accel_sum = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();

                        // Каждые 2 секунды пишем, что мы "живы", даже если не трясем
                        if last_shake.elapsed() > Duration::from_secs(2) {
                            println!("[IDLE] Система работает. Текущий G: {:.2}", accel_sum);
                        }

                        if accel_sum > threshold && last_shake.elapsed() > Duration::from_millis(800) {
                            torch_on = !torch_on;
                            let arg = if torch_on { "on" } else { "off" };
                            
                            Command::new("termux-torch").arg(arg).spawn().ok();
                            
                            println!("!!! [SHAKE] Сила: {:.2} -> Фонарик: {}", accel_sum, arg);
                            last_shake = Instant::now();
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("[ERROR] Ошибка чтения потока: {}", e);
                break;
            }
        }
    }

    println!("--- [END] Программа завершилась сама по себе ---");
}

