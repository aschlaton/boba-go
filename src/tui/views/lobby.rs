use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

type PeerId = u64;

/// Render the host lobby (waiting for players to join)
pub fn render_host_lobby(
    f: &mut Frame,
    game_id: &str,
    host_name: &str,
    connected_players: &[(PeerId, String)],
    expected_players: usize,
) {
    let area = f.area();

    let outer = Block::default()
        .title(Span::styled(
            "Hosting Game",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(outer.clone(), area);
    let size = outer.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Game ID
            Constraint::Length(3), // Status
            Constraint::Min(5),    // Player list
            Constraint::Length(2), // Footer
        ])
        .split(size);

    // Game ID
    let game_id_text = format!("Game ID: {}", game_id);
    let game_id_para = Paragraph::new(game_id_text)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(game_id_para, chunks[0]);

    // Status
    let current = connected_players.len() + 1; // +1 for host
    let status_text = format!("Waiting for players... ({}/{})", current, expected_players);
    let status_para = Paragraph::new(status_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_para, chunks[1]);

    // Player list
    let mut items = vec![ListItem::new(Line::from(vec![
        Span::styled("👑 ", Style::default().fg(Color::Yellow)),
        Span::styled(host_name, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" (Host)"),
    ]))];

    for (i, (_, name)) in connected_players.iter().enumerate() {
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{}. ", i + 2), Style::default().fg(Color::Gray)),
            Span::styled(name, Style::default().fg(Color::White)),
        ])));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Players"));
    f.render_widget(list, chunks[2]);

    // Footer
    let footer = Paragraph::new("Press Q to cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[3]);
}

/// Render the client lobby (waiting to join or waiting for game to start)
pub fn render_client_lobby(
    f: &mut Frame,
    state: &ClientLobbyState,
) {
    let area = f.area();

    let outer = Block::default()
        .title(Span::styled(
            "Join Game",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(outer.clone(), area);
    let size = outer.inner(area);

    match state {
        ClientLobbyState::EnteringName { current_input } => {
            render_name_input(f, size, current_input);
        }
        ClientLobbyState::EnteringHostAddress { current_input } => {
            render_host_address_input(f, size, current_input);
        }
        ClientLobbyState::DiscoveringPeers { discovered, selection_index } => {
            render_peer_discovery(f, size, discovered, *selection_index);
        }
        ClientLobbyState::Connecting => {
            render_connecting(f, size);
        }
        ClientLobbyState::WaitingForStart { player_id } => {
            render_waiting_for_start(f, size, *player_id);
        }
        ClientLobbyState::Rejected { reason } => {
            render_rejected(f, size, reason);
        }
    }
}

pub enum ClientLobbyState {
    EnteringName {
        current_input: String,
    },
    EnteringHostAddress {
        current_input: String,
    },
    DiscoveringPeers {
        discovered: Vec<(PeerId, String, String)>, // PeerId, room_name, host_name
        selection_index: usize,
    },
    Connecting,
    WaitingForStart {
        player_id: usize,
    },
    Rejected {
        reason: String,
    },
}

fn render_name_input(f: &mut Frame, area: Rect, current_input: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let prompt = Paragraph::new("Enter your name:")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(prompt, chunks[0]);

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
}

fn render_host_address_input(f: &mut Frame, area: Rect, current_input: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let prompt = Paragraph::new("Enter host address (e.g., /ip4/127.0.0.1/tcp/24915):")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(prompt, chunks[0]);

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

    let footer = Paragraph::new("Press Enter to connect, Backspace to delete, Q to cancel, M for mDNS discovery")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[2]);
}

fn render_peer_discovery(f: &mut Frame, area: Rect, discovered: &[(PeerId, String, String)], selection_index: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(area);

    let title = Paragraph::new("Discovered Games (mDNS)")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    if discovered.is_empty() {
        let no_peers = Paragraph::new("No games found on local network...\nMake sure host is running!")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(no_peers, chunks[1]);
    } else {
        let items: Vec<ListItem> = discovered
            .iter()
            .enumerate()
            .map(|(i, (_peer_id, room_name, host_name))| {
                let style = if i == selection_index {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let marker = if i == selection_index { "▶ " } else { "  " };

                ListItem::new(Line::from(vec![
                    Span::styled(marker, Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{} ", room_name), style),
                    Span::styled(format!("(hosted by {})", host_name), Style::default().fg(Color::Gray)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Available Rooms"));
        f.render_widget(list, chunks[1]);
    }

    let footer = Paragraph::new("↑/↓ to select, Enter to join, Q to cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[2]);
}

fn render_connecting(f: &mut Frame, area: Rect) {
    let text = Paragraph::new("Connecting to host...")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(text, area);
}

fn render_waiting_for_start(f: &mut Frame, area: Rect, player_id: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let message = Paragraph::new(vec![
        Line::from(Span::styled(
            "✓ Joined Successfully!",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("You are Player {}", player_id + 1)),
        Line::from(""),
        Line::from("Waiting for host to start game..."),
    ])
    .alignment(Alignment::Center);
    f.render_widget(message, chunks[0]);

    let footer = Paragraph::new("Press Q to leave")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[1]);
}

fn render_rejected(f: &mut Frame, area: Rect, reason: &str) {
    let message = Paragraph::new(vec![
        Line::from(Span::styled(
            "✗ Join Rejected",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(reason),
        Line::from(""),
        Line::from("Press any key to return"),
    ])
    .alignment(Alignment::Center);
    f.render_widget(message, area);
}
