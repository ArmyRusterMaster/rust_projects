mod config;
mod generator;
mod random;

use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let config = match config::PasswordConfig::parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Ошибка: {}", e);
            std::process::exit(1);
        }
    };

    let passwords = generator::generate_passwords(&config)?;

    match config.output_file {
        Some(ref filename) => {
            let mut file = File::create(filename)?;
            for pwd in &passwords {
                writeln!(file, "{}", pwd)?;
            }
        }
        None => {
            for pwd in passwords {
                println!("{}", pwd);
            }
        }
    }

    Ok(())
}

