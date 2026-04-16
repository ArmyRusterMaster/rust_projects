use crate::config::PasswordConfig;
use crate::random::{get_random_index, get_random_byte};
use std::io;

const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";
const AMBIGUOUS: &str = "0O1l";

pub fn generate_passwords(config: &PasswordConfig) -> io::Result<Vec<String>> {
    (0..config.count)
        .map(|_| generate_password(config))
        .collect()
}

fn generate_password(config: &PasswordConfig) -> io::Result<String> {
    let alphabet = build_alphabet(config)?;
    let mut password = Vec::with_capacity(config.length);

    if config.ensure {
        ensure_character_types(&mut password, config)?;
    }

    while password.len() < config.length {
        let index = get_random_index(alphabet.len())?;
        password.push(alphabet.chars().nth(index).unwrap());
    }

    if !config.pronounceable {
        shuffle(&mut password)?;
    }

    Ok(password.into_iter().collect())
}

fn build_alphabet(config: &PasswordConfig) -> io::Result<String> {
    let mut alphabet = String::new();
    if config.use_lower { alphabet.push_str(LOWER); }
    if config.use_upper { alphabet.push_str(UPPER); }
    if config.use_digits { alphabet.push_str(DIGITS); }
    if config.use_symbols { alphabet.push_str(SYMBOLS); }

    if config.no_ambiguous {
        alphabet.retain(|c| !AMBIGUOUS.contains(c));
    }

    if alphabet.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Алфавит пуст после фильтрации"));
    }

    Ok(alphabet)
}

fn ensure_character_types(password: &mut Vec<char>, config: &PasswordConfig) -> io::Result<()> {
    // Собираем символы из каждого включённого набора
    let mut required_chars = Vec::new();

    if config.use_lower {
        let lower_chars: Vec<char> = LOWER.chars().collect();
        let idx = get_random_index(lower_chars.len())?;
        required_chars.push(lower_chars[idx]);
    }
    if config.use_upper {
        let upper_chars: Vec<char> = UPPER.chars().collect();
        let idx = get_random_index(upper_chars.len())?;
        required_chars.push(upper_chars[idx]);
    }
    if config.use_digits {
        let digit_chars: Vec<char> = DIGITS.chars().collect();
        let idx = get_random_index(digit_chars.len())?;
        required_chars.push(digit_chars[idx]);
    }
    if config.use_symbols {
        let symbol_chars: Vec<char> = SYMBOLS.chars().collect();
        let idx = get_random_index(symbol_chars.len())?;
        required_chars.push(symbol_chars[idx]);
    }

    // Перемешиваем обязательные символы
    shuffle(&mut required_chars)?;

    // Добавляем их в пароль
    password.extend(required_chars);

    Ok(())
}

fn shuffle(chars: &mut [char]) -> io::Result<()> {
    for i in 0..chars.len() {
        let j = get_random_index(chars.len())?;
        chars.swap(i, j);
    }
    Ok(())
}

