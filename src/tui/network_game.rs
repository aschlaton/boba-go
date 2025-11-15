use std::io;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use tokio::time::sleep;

use crate::engine::GameError;
use crate::network::{Host, Client, lobby::{LobbyHostState, LobbyClientState}};

/// Host a network game
pub async fn run_host_game() -> Result<(), GameError> {
    enable_raw_mode().map_err(|e| GameError::Other(e.to_string()))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| GameError::Other(e.to_string()))?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).map_err(|e| GameError::Other(e.to_string()))?;

    let mut room_name = String::new();
    let mut host_name = String::new();
    let mut input_phase = 0;

    // Input phase
    loop {
        terminal.draw(|f| {
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

            let footer = Paragraph::new("Press Enter to continue, Backspace to delete, Esc to cancel")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(footer, chunks[2]);
        }).map_err(|e| GameError::Other(e.to_string()))?;

        if event::poll(Duration::from_millis(100)).map_err(|e| GameError::Other(e.to_string()))? {
            if let Event::Key(key) = event::read().map_err(|e| GameError::Other(e.to_string()))? {
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
                            disable_raw_mode().ok();
                            execute!(io::stdout(), LeaveAlternateScreen).ok();
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Create host lobby
    let mut lobby = Host::<LobbyHostState>::new(room_name.clone(), host_name.clone()).await
        .map_err(|e| GameError::Other(e.to_string()))?;
    lobby.listen("/ip4/0.0.0.0/tcp/0")
        .map_err(|e| GameError::Other(e.to_string()))?;

    let mut listening_addr = None;

    // Lobby loop
    loop {
        // Poll for network events (non-blocking)
        tokio::select! {
            Some(event) = lobby.next_event() => {
                use crate::network::HostEvent;
                match event {
                    HostEvent::Listening { address } => {
                        listening_addr = Some(address.to_string());
                    }
                    HostEvent::PlayerJoined { .. } => {
                        // Players list updated automatically
                    }
                    HostEvent::PlayerLeft { .. } => {
                        // Players list updated automatically
                    }
                }
            }
            _ = sleep(Duration::from_millis(50)) => {
                // Just continue to render
            }
        }

        // Render
        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(area);

            // Title
            let title = Paragraph::new(format!("Hosting: {}", room_name))
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            // Address
            let addr_text = if let Some(addr) = &listening_addr {
                format!("Address: {}", addr)
            } else {
                "Starting server...".to_string()
            };
            let addr = Paragraph::new(addr_text)
                .style(Style::default().fg(Color::Green))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Connection Info"));
            f.render_widget(addr, chunks[1]);

            // Players
            let players = lobby.get_lobby_players();
            let player_items: Vec<ListItem> = players
                .iter()
                .map(|p| ListItem::new(format!("• {}", p.name)))
                .collect();
            let player_list = List::new(player_items)
                .block(Block::default().borders(Borders::ALL).title("Players in Lobby"));
            f.render_widget(player_list, chunks[2]);

            // Footer
            let footer = Paragraph::new("Press Esc to quit")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(footer, chunks[3]);
        }).map_err(|e| GameError::Other(e.to_string()))?;

        // Handle input
        if event::poll(Duration::from_millis(10)).map_err(|e| GameError::Other(e.to_string()))? {
            if let Event::Key(key) = event::read().map_err(|e| GameError::Other(e.to_string()))? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                    break;
                }
            }
        }
    }

    disable_raw_mode().ok();
    execute!(io::stdout(), LeaveAlternateScreen).ok();
    Ok(())
}

/// Join a network game
pub async fn run_join_game() -> Result<(), GameError> {
    enable_raw_mode().map_err(|e| GameError::Other(e.to_string()))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| GameError::Other(e.to_string()))?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).map_err(|e| GameError::Other(e.to_string()))?;

    let mut player_name = String::new();
    let mut host_address = String::new();
    let mut input_phase = 0;

    // Input phase
    loop {
        terminal.draw(|f| {
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
                "Enter your name:"
            } else {
                "Enter host address (e.g., /ip4/127.0.0.1/tcp/12345):"
            };
            let prompt_para = Paragraph::new(prompt)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(prompt_para, chunks[0]);

            let current_input = if input_phase == 0 { &player_name } else { &host_address };
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

            let footer = Paragraph::new("Press Enter to continue, Backspace to delete, Esc to cancel")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(footer, chunks[2]);
        }).map_err(|e| GameError::Other(e.to_string()))?;

        if event::poll(Duration::from_millis(100)).map_err(|e| GameError::Other(e.to_string()))? {
            if let Event::Key(key) = event::read().map_err(|e| GameError::Other(e.to_string()))? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => {
                            let current = if input_phase == 0 { &mut player_name } else { &mut host_address };
                            if current.len() < 50 {
                                current.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            let current = if input_phase == 0 { &mut player_name } else { &mut host_address };
                            current.pop();
                        }
                        KeyCode::Enter => {
                            let current = if input_phase == 0 { &player_name } else { &host_address };
                            if !current.is_empty() {
                                if input_phase == 0 {
                                    input_phase = 1;
                                } else {
                                    break;
                                }
                            }
                        }
                        KeyCode::Esc => {
                            disable_raw_mode().ok();
                            execute!(io::stdout(), LeaveAlternateScreen).ok();
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Create client lobby and connect
    let mut lobby = Client::<LobbyClientState>::new("default".to_string(), player_name.clone()).await
        .map_err(|e| GameError::Other(e.to_string()))?;
    lobby.connect(&host_address)
        .map_err(|e| GameError::Other(e.to_string()))?;

    let mut status = "Connecting...".to_string();
    let mut connected = false;

    // Lobby loop
    loop {
        // Poll for network events (non-blocking)
        tokio::select! {
            Some(event) = lobby.next_event() => {
                use crate::network::ClientEvent;
                match event {
                    ClientEvent::JoinedLobby { player_id, .. } => {
                        status = format!("Connected! Your ID: {}", player_id);
                        connected = true;
                    }
                    ClientEvent::JoinRejected { reason } => {
                        status = format!("Rejected: {}", reason);
                    }
                    ClientEvent::LobbyUpdated { .. } => {
                        // Players list updated automatically
                    }
                    ClientEvent::Disconnected => {
                        status = "Disconnected from host".to_string();
                        connected = false;
                    }
                    ClientEvent::Error { message } => {
                        status = format!("Error: {}", message);
                    }
                }
            }
            _ = sleep(Duration::from_millis(50)) => {
                // Just continue to render
            }
        }

        // Render
        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(area);

            // Title
            let title = Paragraph::new("Joining Game")
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            // Status
            let status_para = Paragraph::new(status.clone())
                .style(Style::default().fg(if connected { Color::Green } else { Color::Yellow }))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(status_para, chunks[1]);

            // Players
            let players = lobby.get_lobby_players();
            let player_items: Vec<ListItem> = players
                .iter()
                .map(|p| ListItem::new(format!("• {}", p.name)))
                .collect();
            let player_list = List::new(player_items)
                .block(Block::default().borders(Borders::ALL).title("Players in Lobby"));
            f.render_widget(player_list, chunks[2]);

            // Footer
            let footer = Paragraph::new("Press Esc to quit")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(footer, chunks[3]);
        }).map_err(|e| GameError::Other(e.to_string()))?;

        // Handle input
        if event::poll(Duration::from_millis(10)).map_err(|e| GameError::Other(e.to_string()))? {
            if let Event::Key(key) = event::read().map_err(|e| GameError::Other(e.to_string()))? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                    break;
                }
            }
        }
    }

    disable_raw_mode().ok();
    execute!(io::stdout(), LeaveAlternateScreen).ok();
    Ok(())
}
