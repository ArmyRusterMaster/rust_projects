use std::io;

#[derive(Debug)]
pub struct PasswordConfig {
    pub length: usize,
    pub count: usize,
    pub use_lower: bool,
    pub use_upper: bool,
    pub use_digits: bool,
    pub use_symbols: bool,
    pub no_ambiguous: bool,
    pub ensure: bool,
    pub output_file: Option<String>,
    pub pronounceable: bool,
}

impl PasswordConfig {
    pub fn new() -> Self {
        Self {
            length: 16,
            count: 1,
            use_lower: true,
            use_upper: true,
            use_digits: true,
            use_symbols: true,
            no_ambiguous: false,
            ensure: false,
            output_file: None,
            pronounceable: false,
        }
    }

    pub fn parse_args(args: &[String]) -> io::Result<Self> {
        let mut config = Self::new();
        let mut i = 1;

        while i < args.len() {
            match args[i].as_str() {
                "-l" | "--length" => {
                    i += 1;
                    config.length = args.get(i)
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Некорректная длина"))?;
                }
                "-c" | "--count" => {
                    i += 1;
            config.count = args.get(i)
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Некорректное количество"))?;
                }
                "--no-ambiguous" => config.no_ambiguous = true,
                "--ensure" => config.ensure = true,
                "--pronounceable" => config.pronounceable = true,
                "-o" | "--output" => {
            i += 1;
            config.output_file = Some(args.get(i)
                .cloned()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Отсутствует имя файла"))?);
                }
                "--lower" => config.use_lower = true,
                "--upper" => config.use_upper = true,
                "--digits" => config.use_digits = true,
                "--symbols" => config.use_symbols = true,
                "--help" => {
            print_help();
            std::process::exit(0);
                }
                "--version" => {
            println!("passgen v1.0.0");
            std::process::exit(0);
                }
                _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Неизвестный аргумент: {}", args[i]))),
            }
            i += 1;
        }

        // Проверка корректности конфигурации
        if config.length < 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Длина должна быть ≥ 4"));
        }
        if !(config.use_lower || config.use_upper || config.use_digits || config.use_symbols) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Должен быть выбран хотя бы один набор символов"));
        }

        Ok(config)
    }
}

fn print_help() {
    println!("Использование: passgen [OPTIONS]");
    println!("Опции:");
    println!("  -l, --length N        Длина пароля (4–128, по умолчанию 16)");
    println!("  -c, --count N         Количество паролей (по умолчанию 1)");
    println!("  --lower               Включать строчные буквы (a–z)");
    println!("  --upper               Включать заглавные буквы (A–Z)");
    println!("  --digits              Включать цифры (0–9)");
    println!("  --symbols             Включать специальные символы");
    println!("  --no-ambiguous        Исключать похожие символы (0, O, 1, l)");
    println!("  --ensure              Гарантированно включать каждый выбранный тип");
    println!("  --pronounceable       Генерировать произносимые пароли");
    println!("  -o, --output FILE     Сохранить в файл");
    println!("  --help                Показать эту справку");
    println!("  --version             Показать версию");
}

