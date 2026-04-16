use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, ListState},
};
use serde::Deserialize;
use std::{error::Error, io, process::Command};

#[derive(Deserialize, Debug, Clone)]
struct WifiNetwork {
    #[serde(default)]
    bssid: String,
    #[serde(default)]
    frequency_mhz: i32,
    #[serde(default)]
    rssi: i32,
    #[serde(default)]
    ssid: String,
}

fn get_wifi_scan() -> Vec<WifiNetwork> {
    let output = Command::new("termux-wifi-scaninfo")
        .output()
        .expect("Failed to execute termux-wifi-scaninfo");

    serde_json::from_slice(&output.stdout).unwrap_or_default()
}

fn main() -> Result<(), Box<dyn Error>> {
    // Настройка терминала
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut list_state = ListState::default();
    let mut networks = get_wifi_scan();
    list_state.select(Some(0));

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            let items: Vec<ListItem> = networks
                .iter()
                .map(|n| {
                    ListItem::new(format!(
                        "{} | RSSI: {} | BSSID: {}",
                        if n.ssid.is_empty() {
                            "<Hidden>"
                        } else {
                            &n.ssid
                        },
                        n.rssi,
                        n.bssid
                    ))
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title("WiFi Scan (R - refresh, Q - quit)")
                        .borders(Borders::ALL),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[0], &mut list_state);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('r') => {
                        networks = get_wifi_scan();
                        list_state.select(Some(0));
                    }
                    KeyCode::Down => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i >= networks.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        list_state.select(Some(i));
                    }
                    KeyCode::Up => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    networks.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        list_state.select(Some(i));
                    }
                    _ => {}
                }
            }
        }
    }

    // Восстановление терминала
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
