mod network;
mod scanner;
mod ui;
mod events;

use crate::ui::{AppState, InputMode};
use crate::events::EventHandler;
use crossterm::{event::{self, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use std::{io, time::Duration, net::IpAddr};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut state = AppState {
        target_input: String::new(),
        ports_input: String::from("1-1000"),
        input_mode: InputMode::EditingTarget,
        results: Vec::new(),
        progress: 0.0,
        status: String::from("Tab: переключение | Enter: Старт | f: Фильтр | c: Очистить"),
        list_state: ListState::default(),
        count_open: 0,
        count_closed: 0,
        filter_open_only: false,
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<(IpAddr, u16, bool)>();
    let mut total_tasks: f64 = 1.0;
    let mut scanned_count: f64 = 0.0;

    loop {
        ui::draw(&mut terminal, &mut state)?;

        while let Ok((ip, port, is_open)) = rx.try_recv() {
            let status = if is_open { state.count_open += 1; "OPEN".to_string() } 
                         else { state.count_closed += 1; "CLOSED".to_string() };
            state.results.push((ip, port, status));
            scanned_count += 1.0;
            state.progress = (scanned_count / total_tasks).min(1.0);
            state.results.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        }

        if event::poll(Duration::from_millis(20))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('f') | KeyCode::Char('а') => state.filter_open_only = !state.filter_open_only,
                    KeyCode::Char('c') | KeyCode::Char('с') => {
                        state.results.clear();
                        state.count_open = 0;
                        state.count_closed = 0;
                        state.progress = 0.0;
                    }
                    KeyCode::Enter if state.input_mode != InputMode::Scanning => {
                        if let Ok(tr) = network::TargetResolver::new() {
                            let ports = network::TargetResolver::parse_ports(&state.ports_input);
                            let ips = tr.resolve(&state.target_input).await.unwrap_or_default();
                            if !ips.is_empty() && !ports.is_empty() {
                                total_tasks = (ips.len() * ports.len()) as f64;
                                scanned_count = 0.0;
                                state.results.clear();
                                state.input_mode = InputMode::Scanning;
                                let tx_c = tx.clone();
                                tokio::spawn(async move {
                                    let scanner = scanner::PortScanner::new(200, 500);
                                    scanner.scan(ips, ports, tx_c).await;
                                });
                            }
                        }
                    }
                    _ => { if EventHandler::handle_key(key, &mut state) { break; } }
                }
            }
        }
        if state.input_mode == InputMode::Scanning && state.progress >= 1.0 {
            state.input_mode = InputMode::Finished;
            state.status = String::from("Готово! f: фильтр | c: сброс | q: выход");
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

