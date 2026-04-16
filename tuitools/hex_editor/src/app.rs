use memmap2::Mmap;
use std::io::{Seek, SeekFrom, Write};
use std::{collections::BTreeMap, fs::File, path::PathBuf};
pub struct App {
    pub mmap: Mmap,
    pub cursor: u64,
    pub scroll: u64,
    pub rows: u16,
    pub cols: u16,
    pub modifications: BTreeMap<u64, u8>,
}

impl App {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self {
            mmap,
            cursor: 0,
            scroll: 0,
            rows: 0,
            cols: 16,
            modifications: BTreeMap::new(),
        })
    }

    pub fn edit_byte(&mut self, hex_char: char) {
        let val = hex_char.to_digit(16).unwrap_or(0) as u8;
        let current_byte = self
            .modifications
            .get(&self.cursor)
            .cloned()
            .unwrap_or(self.mmap[self.cursor as usize]);

        // Логика: сначала меняем старшие 4 бита, потом младшие (или наоборот)
        // Для простоты: просто заменяем весь байт новым значением (или сдвигаем)
        // Упростим: ввод символа заменяет байт целиком для MVP
        self.modifications.insert(self.cursor, val);
    }

    pub fn save_changes(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        if self.modifications.is_empty() {
            return Ok(());
        }

        // Открываем файл на чтение/запись
        let mut file = std::fs::OpenOptions::new().write(true).open(path)?;

        for (offset, byte) in &self.modifications {
            file.seek(SeekFrom::Start(*offset))?;
            file.write_all(&[*byte])?;
        }

        self.modifications.clear(); // Очищаем после записи
        // Переинициализируем mmap, чтобы видеть обновленный файл
        self.mmap = unsafe { memmap2::Mmap::map(&file)? };
        Ok(())
    }

    pub fn move_cursor(&mut self, delta: i64) {
        let new_pos = self.cursor as i64 + delta;
        if new_pos >= 0 && new_pos < self.mmap.len() as i64 {
            self.cursor = new_pos as u64;
        }
    }

    pub fn adjust_scroll(&mut self) {
        let cursor_row = self.cursor / self.cols as u64;
        if cursor_row < self.scroll {
            self.scroll = cursor_row;
        } else if cursor_row >= self.scroll + self.rows as u64 {
            self.scroll = cursor_row - self.rows as u64 + 1;
        }
    }
}
