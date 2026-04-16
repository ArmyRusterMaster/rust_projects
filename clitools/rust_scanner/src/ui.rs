use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::net::IpAddr;

#[derive(PartialEq, Clone, Copy)]
pub enum InputMode { EditingTarget, EditingPorts, Scanning, Finished }

pub struct AppState {
    pub target_input: String,
    pub ports_input: String,
    pub input_mode: InputMode,
    pub results: Vec<(IpAddr, u16, String)>,
    pub progress: f64,
    pub status: String,
    pub list_state: ListState,
    pub count_open: u32,
    pub count_closed: u32,
    pub filter_open_only: bool,
}

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, state: &mut AppState) -> anyhow::Result<()> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Поля ввода
                Constraint::Length(3), // Прогресс
                Constraint::Min(5),    // Список
                Constraint::Length(3), // Статус
            ])
            .split(f.area());

        let input_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        // Поля ввода
        let target_style = if state.input_mode == InputMode::EditingTarget { Color::Yellow } else { Color::White };
        f.render_widget(Paragraph::new(state.target_input.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Target ").border_style(Style::default().fg(target_style))), 
            input_chunks[0]);

        let ports_style = if state.input_mode == InputMode::EditingPorts { Color::Yellow } else { Color::White };
        f.render_widget(Paragraph::new(state.ports_input.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Ports ").border_style(Style::default().fg(ports_style))), 
            input_chunks[1]);

        // Прогресс
        let label = format!("{:.1}% (Open: {} / Closed: {})", state.progress * 100.0, state.count_open, state.count_closed);
        f.render_widget(Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Progress "))
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(state.progress)
            .label(label), chunks[1]);

        // Список с фильтрацией
        let filter_title = if state.filter_open_only { " [ Фильтр: ОТКРЫТЫЕ ] " } else { " [ Фильтр: ВСЕ ] " };
        let items: Vec<ListItem> = state.results.iter()
            .filter(|(_, _, status)| !state.filter_open_only || status == "OPEN")
            .map(|(ip, port, status)| {
                let color = if status == "OPEN" { Color::Green } else { Color::Red };
                ListItem::new(format!("{:<15} : {:<5} -> {}", ip, port, status)).style(Style::default().fg(color))
            })
            .collect();

        f.render_stateful_widget(
            List::new(items)
                .block(Block::default().borders(Borders::ALL).title(format!(" Results{} ", filter_title)))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Indexed(236)))
                .highlight_symbol(">> "),
            chunks[2],
            &mut state.list_state
        );

        // Статус
        f.render_widget(Paragraph::new(state.status.as_str()).block(Block::default().borders(Borders::ALL)), chunks[3]);

    }).map_err(|e| anyhow::anyhow!("Draw error: {}", e))?;
    Ok(())
}

