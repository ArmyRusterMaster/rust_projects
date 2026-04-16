use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(f: &mut Frame, app: &mut App) {
    // 1. Разбиваем экран: Основная область и Статус-бар (1 строка внизу)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Hex-редактор забирает всё место
            Constraint::Length(1), // Статус-бар
        ])
        .split(f.size());

    // Рассчитываем количество видимых строк (высота минус рамки блока)
    app.rows = chunks[0].height.saturating_sub(2);
    app.adjust_scroll();

    let mut lines = Vec::new();
    let start_byte = app.scroll * app.cols as u64;

    // 2. Формируем строки данных
    for r in 0..app.rows {
        let row_addr = start_byte + (r as u64 * app.cols as u64);
        if row_addr >= app.mmap.len() as u64 {
            break;
        }

        let mut row_spans = Vec::new();

        // --- Колонка адреса (Offset) ---
        row_spans.push(Span::styled(
            format!("{:08X}: ", row_addr),
            Style::default().fg(Color::DarkGray),
        ));

        // --- Колонка Hex-байтов ---
        let mut ascii_preview = String::new();
        for c in 0..app.cols {
            let addr = row_addr + c as u64;

            if addr < app.mmap.len() as u64 {
                let byte = app.mmap[addr as usize];

                // Проверяем наличие правок в BTreeMap (на будущее)
                let is_modified = app.modifications.contains_key(&addr);
                let display_byte = *app.modifications.get(&addr).unwrap_or(&byte);

                // Стиль байта
                let mut style = Style::default();
                if addr == app.cursor {
                    style = style
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD);
                } else if is_modified {
                    style = style.fg(Color::Red); // Измененные байты — красные
                } else if display_byte == 0 {
                    style = style.fg(Color::DarkGray); // Нули — серые
                } else {
                    // Обычный байт
                    style = style.fg(Color::White)
                }
                row_spans.push(Span::styled(format!("{:02X} ", display_byte), style));

                // Формируем ASCII символ
                let ch = if display_byte.is_ascii_graphic() {
                    display_byte as char
                } else {
                    '.'
                };
                ascii_preview.push(ch);
            } else {
                row_spans.push(Span::raw("   ")); // Заполнитель для пустых мест в конце файла
                ascii_preview.push(' ');
            }
        }

        // --- Разделитель и ASCII панель ---
        row_spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
        row_spans.push(Span::styled(
            ascii_preview,
            Style::default().fg(Color::Cyan),
        ));

        lines.push(Line::from(row_spans));
    }

    // 3. Отрисовка основного виджета
    let hex_widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Hex Editor (Rust) ")
            .title_alignment(ratatui::layout::Alignment::Center),
    );
    f.render_widget(hex_widget, chunks[0]);

    // 4. Отрисовка статус-бара
    let mode = " [VIEW MODE] ";
    let status_text = format!(
        "{} Offset: 0x{:08X} | Total: {} bytes | Q: Quit",
        mode,
        app.cursor,
        app.mmap.len()
    );

    let status_bar =
        Paragraph::new(status_text).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(status_bar, chunks[1]);
}
