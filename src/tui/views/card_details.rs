use crate::engine::CardKind;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_card_details(
    f: &mut Frame,
    area: Rect,
    card_kind: Option<CardKind>,
) {
    if let Some(card_kind) = card_kind {
        // Create the outer box with card name as title
        let card_box = Block::default()
            .borders(Borders::ALL)
            .title(card_kind.name());
        let inner_area = card_box.inner(area);
        f.render_widget(card_box, area);
        
        // Calculate description height needed
        let description = card_kind.description();
        let flavor_text = card_kind.flavor_text();
        
        // Count lines needed for description (with wrapping)
        let desc_lines = description.chars().count() as u16 / inner_area.width.max(1) + 1;
        let has_flavor = !flavor_text.is_empty() && flavor_text != "placeholder";
        let flavor_lines = if has_flavor { 1 } else { 0 };
        let blank_line = if has_flavor { 1 } else { 0 };
        let border_lines = 2; // top and bottom border
        let title_line = 1; // title takes 1 line
        let desc_height = desc_lines + flavor_lines + blank_line + border_lines + title_line;
        
        // Split inner area: art takes remaining space, description takes what it needs
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // Art area (top) - takes remaining space
                Constraint::Length(desc_height), // Description/flavor text - takes needed space
            ])
            .split(inner_area);
        
        // Art area (top) - empty for now
        let art_para = Paragraph::new("");
        f.render_widget(art_para, inner_chunks[0]);

        // Description and flavor text - inside the card box with its own border
        let mut text_lines = Vec::new();
        
        // Add description
        text_lines.push(Line::from(description));
        
        // Add blank line and flavor text if present
        if has_flavor {
            text_lines.push(Line::from(""));
            let quoted_flavor = format!("\"{}\"", flavor_text);
            let flavor_line = Line::from(vec![
                ratatui::text::Span::styled(
                    quoted_flavor,
                    Style::default().add_modifier(Modifier::ITALIC)
                )
            ])
            .alignment(Alignment::Center);
            text_lines.push(flavor_line);
        }
        
        let text_para = Paragraph::new(ratatui::text::Text::from(text_lines))
            .block(Block::default().borders(Borders::ALL).title("Description"))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(text_para, inner_chunks[1]);
    } else {
        // Empty state
        let empty_box = Block::default()
            .borders(Borders::ALL)
            .title("Card Details");
        f.render_widget(empty_box, area);
    }
}

