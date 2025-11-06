use crate::engine::Game;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_my_cards(
    f: &mut Frame,
    game: &Game,
    current_player_id: usize,
    area: Rect,
) {
    let player_public = match game.get_player_public(current_player_id) {
        Ok(p) => p,
        Err(_) => {
            let error_para = Paragraph::new("Error loading cards")
                .block(Block::default().borders(Borders::ALL).title("My Cards"));
            f.render_widget(error_para, area);
            return;
        }
    };
    
    // Format public cards
    let mut card_texts = Vec::new();
    for (card_kind, count) in &player_public.public_cards {
        if *count > 0 {
            card_texts.push(format!("{}x {}", count, card_kind.name()));
        }
    }
    
    // Format boosted fruit teas
    for (card_kind, count) in &player_public.boosted_fruit_teas {
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
            format!("{}'s Cards", player_public.name),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
    f.render_widget(para, area);
}

