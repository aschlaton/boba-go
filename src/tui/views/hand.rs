use std::collections::HashMap;
use crate::engine::{Game, CardKind};
use super::card_details::render_card_details;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render_hand(
    f: &mut Frame,
    game: &Game,
    current_player_id: usize,
    area: Rect,
    selected_cards: Option<&HashMap<CardKind, usize>>,
    highlight_index: usize,
) {
    // Split area: 60% hand list, 40% card details
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

    let hand_result = game.get_player_hand(current_player_id);
    let hand = match hand_result {
        Ok(h) => h,
        Err(_) => {
            let error_para = Paragraph::new("Error loading hand")
                .block(Block::default().borders(Borders::ALL).title("Your Hand"));
            f.render_widget(error_para, chunks[0]);
            return;
        }
    };
    
    let hand_vec: Vec<(CardKind, usize)> = hand.iter()
        .filter(|(_, count)| **count > 0)
        .map(|(k, v)| (*k, *v))
        .collect();
    
    let mut items = Vec::new();
    for (idx, (card_kind, count)) in hand_vec.iter().enumerate() {
        let card_name = card_kind.name();
        let score = card_kind.score();
        
        let selected_count = selected_cards
            .and_then(|sc| sc.get(card_kind).copied())
            .unwrap_or(0);
        
        let text = if selected_count > 0 {
            format!("{}x {} ({} pts) [Selected: {}]", count, card_name, score, selected_count)
        } else {
            format!("{}x {} ({} pts)", count, card_name, score)
        };
        
        let style: Style = if idx == highlight_index {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        
        items.push(ListItem::new(Line::from(vec![ratatui::text::Span::styled(text, style)])));
    }
    
    if items.is_empty() {
        items.push(ListItem::new("No cards in hand"));
    }
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Your Hand"))
        .highlight_style(Style::default().bg(Color::Rgb(180, 180, 180)))
        .highlight_symbol("▶ ");
    f.render_widget(list, chunks[0]);

    // Card details box (right side)
    let highlighted_card = if !hand_vec.is_empty() && highlight_index < hand_vec.len() {
        Some(hand_vec[highlight_index].0)
    } else {
        None
    };
    render_card_details(f, chunks[1], highlighted_card);
}

