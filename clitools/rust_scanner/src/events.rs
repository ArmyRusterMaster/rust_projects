use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::{AppState, InputMode};

pub struct EventHandler;

impl EventHandler {
    pub fn handle_key(key: KeyEvent, state: &mut AppState) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            
            KeyCode::Up => {
                if !state.results.is_empty() {
                    let i = match state.list_state.selected() {
                        Some(i) => if i == 0 { state.results.len().saturating_sub(1) } else { i - 1 },
                        None => 0,
                    };
                    state.list_state.select(Some(i));
                }
            }
            KeyCode::Down => {
                if !state.results.is_empty() {
                    let i = match state.list_state.selected() {
                        Some(i) => if i >= state.results.len().saturating_sub(1) { 0 } else { i + 1 },
                        None => 0,
                    };
                    state.list_state.select(Some(i));
                }
            }

            KeyCode::Tab => {
                state.input_mode = match state.input_mode {
                    InputMode::EditingTarget => InputMode::EditingPorts,
                    InputMode::EditingPorts => InputMode::EditingTarget,
                    _ => InputMode::EditingTarget,
                };
            }

            KeyCode::Char(c) => {
                match state.input_mode {
                    InputMode::EditingTarget => state.target_input.push(c),
                    InputMode::EditingPorts => state.ports_input.push(c),
                    _ => {}
                }
            }
            KeyCode::Backspace => {
                match state.input_mode {
                    InputMode::EditingTarget => { state.target_input.pop(); }
                    InputMode::EditingPorts => { state.ports_input.pop(); }
                    _ => {}
                }
            }
            _ => {}
        }
        false
    }
}

