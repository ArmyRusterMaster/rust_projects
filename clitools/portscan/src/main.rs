use clap::Parser;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use std::net::{IpAddr, SocketAddr};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    target: IpAddr,
    #[arg(short, long, default_value_t = 1)]
    start: u16,
    #[arg(short, long, default_value_t = 1000)]
    end: u16,
    #[arg(short, long, default_value_t = 100)] // Ограничим количество одновременных попыток
    concurrency: usize,
}

async fn scan_port(target: IpAddr, port: u16, pb: ProgressBar) -> Option<u16> {
    let addr = SocketAddr::new(target, port);
    let res = timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await;
    
    pb.inc(1); // Двигаем прогрессбар сразу после попытки
    
    match res {
        Ok(Ok(_)) => Some(port),
        _ => None,
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let total_ports = (args.end - args.start + 1) as u64;

    let pb = ProgressBar::new(total_ports);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut tasks = Vec::new();

    // Создаем спавны для всех портов
    for port in args.start..=args.end {
        let pb_clone = pb.clone();
        let target = args.target;
        
        // tokio::spawn отправляет задачу в пул потоков
        tasks.push(tokio::spawn(async move {
            scan_port(target, port, pb_clone).await
        }));
    }

    // Ждем выполнения всех задач
    let results = join_all(tasks).await;
    pb.finish_with_message("Сканирование завершено");

    let mut open_ports: Vec<u16> = results
        .into_iter()
        .filter_map(|res| res.ok().flatten()) // Очищаем от ошибок спавна и пустых портов
        .collect();

    open_ports.sort();

    if open_ports.is_empty() {
        println!("Открытых портов не найдено.");
    } else {
        println!("\n--- Найдено {} открытых портов ---", open_ports.len());
        for port in open_ports {
            println!("Порт {} [OPEN]", port);
        }
    }
}

