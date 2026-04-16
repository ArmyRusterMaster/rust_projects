use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

pub fn handle_key_events(key: KeyEvent, app: &mut App, path: &std::path::Path) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }

    match key.code {
        KeyCode::Char('q') => return true,

        // Сохранение по нажатию 's'
        KeyCode::Char('s') => {
            let _ = app.save_changes(path);
        }

        // Редактирование (0-9, a-f)
        KeyCode::Char(c) if c.is_ascii_hexdigit() => {
            app.edit_byte(c);
            app.move_cursor(1); // Авто-переход к следующему байту
        }

        // Навигация (стрелки...)
        KeyCode::Up => app.move_cursor(-(app.cols as i64)),
        KeyCode::Down => app.move_cursor(app.cols as i64),
        KeyCode::Left => app.move_cursor(-1),
        KeyCode::Right => app.move_cursor(1),
        _ => {}
    }
    false
}
