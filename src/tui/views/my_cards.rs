use crate::engine::{Game, CardKind};
use super::card_details::render_card_details;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render_my_cards(
    f: &mut Frame,
    game: &Game,
    current_player_id: usize,
    area: Rect,
    highlight_index: usize,
) {
    // Split area: 60% card list, 40% card details
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

    let player_public = match game.get_player_public(current_player_id) {
        Ok(p) => p,
        Err(_) => {
            let error_para = ratatui::widgets::Paragraph::new("Error loading cards")
                .block(Block::default().borders(Borders::ALL).title("My Cards"));
            f.render_widget(error_para, chunks[0]);
            return;
        }
    };
    
    // Build list of cards (public cards + boosted fruit teas)
    let mut card_list: Vec<(CardKind, usize, bool)> = Vec::new();
    
    // Add public cards
    for (card_kind, count) in &player_public.public_cards {
        if *count > 0 {
            card_list.push((*card_kind, *count, false));
        }
    }
    
    // Add boosted fruit teas
    for (card_kind, count) in &player_public.boosted_fruit_teas {
        if *count > 0 {
            card_list.push((*card_kind, *count, true));
        }
    }
    
    // Create list items
    let mut items = Vec::new();
    for (idx, (card_kind, count, is_boosted)) in card_list.iter().enumerate() {
        let card_name = card_kind.name();
        let text = if *is_boosted {
            format!("{}x {} (3x)", count, card_name)
        } else {
            format!("{}x {}", count, card_name)
        };
        
        let style: Style = if idx == highlight_index {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        
        items.push(ListItem::new(Line::from(vec![ratatui::text::Span::styled(text, style)])));
    }
    
    if items.is_empty() {
        items.push(ListItem::new("No cards collected yet"));
    }
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!("{}'s Cards", player_public.name)))
        .highlight_style(Style::default().bg(Color::Rgb(180, 180, 180)))
        .highlight_symbol("▶ ");
    f.render_widget(list, chunks[0]);

    // Card details box (right side)
    let highlighted_card = if !card_list.is_empty() && highlight_index < card_list.len() {
        Some(card_list[highlight_index].0)
    } else {
        None
    };
    render_card_details(f, chunks[1], highlighted_card);
}

