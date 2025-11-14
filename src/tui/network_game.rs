use std::io;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::engine::GameError;
use super::views::{render_host_lobby, render_client_lobby, ClientLobbyState};

/// Host a network game
pub fn run_host_game() -> Result<(), GameError> {
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).expect("create terminal");

    let mut room_name = String::new();
    let mut host_name = String::new();
    let mut input_phase = 0;

    loop {
        let _ = terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(area);

            let prompt = if input_phase == 0 {
                "Enter room name:"
            } else {
                "Enter your name:"
            };
            let prompt_para = Paragraph::new(prompt)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(prompt_para, chunks[0]);

            let current_input = if input_phase == 0 { &room_name } else { &host_name };
            let input_display = if current_input.is_empty() {
                "_".to_string()
            } else {
                format!("{}_", current_input)
            };
            let input = Paragraph::new(input_display)
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(input, chunks[1]);

            let footer = Paragraph::new("Press Enter to continue, Backspace to delete, Q to cancel")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(footer, chunks[2]);
        });

        if let Ok(true) = event::poll(std::time::Duration::from_millis(100)) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => {
                            let current = if input_phase == 0 { &mut room_name } else { &mut host_name };
                            if current.len() < 20 {
                                current.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            let current = if input_phase == 0 { &mut room_name } else { &mut host_name };
                            current.pop();
                        }
                        KeyCode::Enter => {
                            let current = if input_phase == 0 { &room_name } else { &host_name };
                            if !current.is_empty() {
                                if input_phase == 0 {
                                    input_phase = 1;
                                } else {
                                    break;
                                }
                            }
                        }
                        KeyCode::Esc => {
                            let _ = disable_raw_mode();
                            let _ = execute!(io::stdout(), LeaveAlternateScreen);
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let game_id = "TODO TODO TODO";
    let connected_players: Vec<(u64, String)> = Vec::new();
    let expected_players = 2;

    loop {
        let _ = terminal.draw(|f| {
            render_host_lobby(
                f,
                game_id,
                &host_name,
                &connected_players,
                expected_players,
            );
        });

        if let Ok(true) = event::poll(std::time::Duration::from_millis(100)) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                        break;
                    }
                }
            }
        }
    }

    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    Ok(())
}

/// Join a network game
#[allow(unused_assignments)]
pub fn run_join_game() -> Result<(), GameError> {
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).expect("create terminal");

    let mut lobby_state = ClientLobbyState::EnteringName {
        current_input: String::new(),
    };

    #[allow(unused_variables)]
    let mut player_name = String::new();

    loop {
        let _ = terminal.draw(|f| {
            render_client_lobby(f, &lobby_state);
        });

        if let Ok(true) = event::poll(std::time::Duration::from_millis(100)) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    match &mut lobby_state {
                        ClientLobbyState::EnteringName { current_input } => {
                            match key.code {
                                KeyCode::Char('m') | KeyCode::Char('M') => {
                                    if !current_input.is_empty() {
                                        player_name = current_input.clone();
                                        lobby_state = ClientLobbyState::DiscoveringPeers {
                                            discovered: Vec::new(),
                                            selection_index: 0,
                                        };
                                    }
                                }
                                KeyCode::Char(c) => {
                                    if current_input.len() < 20 {
                                        current_input.push(c);
                                    }
                                }
                                KeyCode::Backspace => {
                                    current_input.pop();
                                }
                                KeyCode::Enter => {
                                    if !current_input.is_empty() {
                                        player_name = current_input.clone();
                                        lobby_state = ClientLobbyState::EnteringHostAddress {
                                            current_input: String::new(),
                                        };
                                    }
                                }
                                KeyCode::Esc => {
                                    let _ = disable_raw_mode();
                                    let _ = execute!(io::stdout(), LeaveAlternateScreen);
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }
                        ClientLobbyState::EnteringHostAddress { current_input } => {
                            match key.code {
                                KeyCode::Char('m') | KeyCode::Char('M') => {
                                    lobby_state = ClientLobbyState::DiscoveringPeers {
                                        discovered: Vec::new(),
                                        selection_index: 0,
                                    };
                                }
                                KeyCode::Char(c) => {
                                    current_input.push(c);
                                }
                                KeyCode::Backspace => {
                                    current_input.pop();
                                }
                                KeyCode::Enter => {
                                    if !current_input.is_empty() {
                                        lobby_state = ClientLobbyState::Connecting;
                                    }
                                }
                                KeyCode::Esc => {
                                    let _ = disable_raw_mode();
                                    let _ = execute!(io::stdout(), LeaveAlternateScreen);
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }
                        ClientLobbyState::DiscoveringPeers { discovered, selection_index } => {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    let _ = disable_raw_mode();
                                    let _ = execute!(io::stdout(), LeaveAlternateScreen);
                                    return Ok(());
                                }
                                KeyCode::Up => {
                                    if !discovered.is_empty() {
                                        *selection_index = selection_index.checked_sub(1).unwrap_or(discovered.len() - 1);
                                    }
                                }
                                KeyCode::Down => {
                                    if !discovered.is_empty() {
                                        *selection_index = (*selection_index + 1) % discovered.len();
                                    }
                                }
                                KeyCode::Enter => {
                                    if !discovered.is_empty() && *selection_index < discovered.len() {
                                        lobby_state = ClientLobbyState::Connecting;
                                    }
                                }
                                _ => {}
                            }
                        }
                        ClientLobbyState::Connecting => {
                            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                                let _ = disable_raw_mode();
                                let _ = execute!(io::stdout(), LeaveAlternateScreen);
                                return Ok(());
                            }
                        }
                        ClientLobbyState::WaitingForStart { .. } => {
                            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                                let _ = disable_raw_mode();
                                let _ = execute!(io::stdout(), LeaveAlternateScreen);
                                return Ok(());
                            }
                        }
                        ClientLobbyState::Rejected { .. } => {
                            let _ = disable_raw_mode();
                            let _ = execute!(io::stdout(), LeaveAlternateScreen);
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}
