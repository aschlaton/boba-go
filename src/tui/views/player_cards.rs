use crate::tui::GameInterface;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render_player_cards<G: GameInterface>(
    f: &mut Frame,
    game: &G,
    viewing_player_id: usize,
    area: Rect,
    player_list_index: usize,
) {
    let players_public = game.get_players_public();
    
    // Split area: player list on left, cards on right
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(30),
            ratatui::layout::Constraint::Percentage(70),
        ])
        .split(area);
    
    // Player list
    let mut player_items = Vec::new();
    for (idx, player) in players_public.iter().enumerate() {
        let style = if idx == player_list_index {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        player_items.push(ListItem::new(Span::styled(player.name.clone(), style)));
    }
    
    let player_list = List::new(player_items)
        .block(Block::default().borders(Borders::ALL).title("Players"))
        .highlight_style(Style::default().bg(Color::Rgb(180, 180, 180)))
        .highlight_symbol("▶ ");
    f.render_widget(player_list, chunks[0]);
    
    // Selected player's cards
    if let Some(player) = players_public.get(viewing_player_id) {
        let mut card_texts = Vec::new();
        for (card_kind, count) in &player.public_cards {
            if *count > 0 {
                card_texts.push(format!("{}x {}", count, card_kind.name()));
            }
        }
        
        for (card_kind, count) in &player.boosted_fruit_teas {
            if *count > 0 {
                card_texts.push(format!("{}x {} (3x)", count, card_kind.name()));
            }
        }
        
        let cards_str = if card_texts.is_empty() {
            "No cards collected yet".to_string()
        } else {
            card_texts.join("\n")
        };
        
        let para = Paragraph::new(cards_str)
            .block(Block::default().borders(Borders::ALL).title(Span::styled(
                format!("{}'s Cards", player.name),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )));
        f.render_widget(para, chunks[1]);
    } else {
        let error_para = Paragraph::new("Invalid player")
            .block(Block::default().borders(Borders::ALL).title("Error"));
        f.render_widget(error_para, chunks[1]);
    }
}

