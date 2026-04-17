use crate::ui::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Заголовок
    let title = Paragraph::new(" 🌑 RustShadow | Traffic Monitor ")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Таблица
    let header_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let header =
        Row::new(vec!["IP Address", "Protocol", "Target Domain", "Traffic"]).style(header_style);

    let rows = app.connections.iter().map(|c| {
        Row::new(vec![
            c.ip.clone(),
            c.protocol.clone(),
            c.domain.clone(),
            format!("{:.2} MB", c.data_transferred as f64 / 1_048_576.0),
        ])
        .style(Style::default().fg(Color::White))
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(45),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Active Connections ")
            .borders(Borders::ALL),
    )
    .column_spacing(1);

    f.render_widget(table, chunks[1]);

    // Футер
    let footer = Paragraph::new(" [Q] Quit | [S] Settings | Status: Capturing... ")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
