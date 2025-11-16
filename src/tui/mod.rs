use std::io;
use std::collections::HashMap;
use crate::engine::{Game, GameConfig, GameError, ScoreBreakdown, PlayerTurnState, CardKind};

mod views;
mod network_game;
mod game_ui;
mod input;

pub use network_game::{run_host_game, run_join_game};
pub use game_ui::{GameView, GameUIState, GameInterface, render_game_ui};
pub use input::{handle_game_input, calculate_max_selections, InputAction};

#[derive(Copy, Clone)]
pub enum StartAction {
    NewLocalGame,
    HostNetworkGame,
    JoinNetworkGame,
    HowToPlay,
    Quit,
}


const LEFT_ASCII: &str = include_str!("assets/title.txt");
const RIGHT_ASCII: &str = include_str!("assets/boba_art.txt");

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Table, Row, Cell},
    Frame, Terminal,
};

pub fn run_start_page() -> StartAction {
    // Setup terminal
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).expect("create terminal");

    let mut selected: usize = 0;
    let options = [
        ("Start new local game", StartAction::NewLocalGame),
        ("Host network game", StartAction::HostNetworkGame),
        ("Join network game", StartAction::JoinNetworkGame),
        ("How to play", StartAction::HowToPlay),
        ("Quit", StartAction::Quit),
    ];

    let result = loop {
        let _ = terminal.draw(|f| {
            let area = f.area();
            // Outer frame around everything
            let outer = Block::default()
                .title(Span::styled(
                    "Main Menu",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            f.render_widget(outer.clone(), area);
            let size = outer.inner(area);

            // Dynamic title height: leave room for menu+footer
            let title_lines = LEFT_ASCII.lines().count().max(RIGHT_ASCII.lines().count()) as u16;
            let max_title = size.height.saturating_sub(4);
            let title_height = title_lines.min(max_title);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(title_height),
                    Constraint::Min(3),
                    Constraint::Length(2),
                ])
                .split(size);

            // Title row: split into left/right with dynamic widths so content isn't cut
            let left_w: u16 = LEFT_ASCII.lines().map(|l| l.chars().count() as u16).max().unwrap_or(30);
            let right_w: u16 = RIGHT_ASCII.lines().map(|l| l.chars().count() as u16).max().unwrap_or(30);
            let total_w = left_w.saturating_add(right_w).max(1);
            // compute percentages; ensure at least 10%/10%
            let left_pct = ((left_w as u32 * 100) / (total_w as u32)).clamp(10, 90) as u16;
            let right_pct = 100u16.saturating_sub(left_pct);
            let title_row = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(left_pct),
                    Constraint::Percentage(right_pct),
                ])
                .split(chunks[0]);

            // Bottom-justify the shorter art by padding with leading newlines
            let left_lines_ct = LEFT_ASCII.lines().count();
            let right_lines_ct = RIGHT_ASCII.lines().count();
            let pad_lines = right_lines_ct.saturating_sub(left_lines_ct);
            let padded_left = if pad_lines > 0 {
                let mut s = String::new();
                for _ in 0..pad_lines { s.push('\n'); }
                s.push_str(LEFT_ASCII);
                s
            } else {
                LEFT_ASCII.to_string()
            };

            let left = Paragraph::new(Text::from(padded_left))
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Left)
                .block(Block::default());
            f.render_widget(left, title_row[0]);

            // Colorize boba art with brown-sugar streaks on a mostly white cup
            let streak_dark = Color::Rgb(90, 56, 38);
            let streak_med = Color::Rgb(153, 100, 60);
            let streak_light = Color::Rgb(210, 160, 100);
            let cup_white = Color::White;

            let right_lines_raw: Vec<&str> = RIGHT_ASCII.lines().collect();
            let total_lines = right_lines_raw.len().max(1);

            // Detect where the drink (solid fill) actually starts: first line with enough solid blocks
            let is_solid = |c: char| matches!(c, '⣿' | '⣷' | '⣾');
            let mut drink_top_row = total_lines / 2; // fallback
            for (i, ln) in right_lines_raw.iter().enumerate() {
                let solid_count = ln.chars().filter(|&c| is_solid(c)).count();
                if solid_count >= 5 { // threshold heuristic
                    drink_top_row = i;
                    break;
                }
            }

            let right_lines: Vec<Line> = right_lines_raw
                .iter()
                .enumerate()
                .map(|(row, ln)| {
                    let mut spans: Vec<Span> = Vec::with_capacity(ln.chars().count());
                    for (col, ch) in ln.chars().enumerate() {
                        // Only color interior "solid" cup fill glyphs; leave borders/other glyphs uncolored
                        if is_solid(ch) {
                            let color = if row < drink_top_row {
                                // Above drink: mostly white with very faint drips
                                let h = (row as u32).wrapping_mul(13) ^ (col as u32).wrapping_mul(17);
                                if h % 47 == 0 { streak_light } else { cup_white }
                            } else {
                                // Within drink: cloud-like blobs using simple 2D hash-based density
                                let drink_height = total_lines.saturating_sub(drink_top_row).max(1);
                                let max_depth = (drink_height as f32 * 0.38).ceil() as usize; // only top ~38% of drink has syrup
                                let depth = row.saturating_sub(drink_top_row);
                                if depth > max_depth { cup_white } else {
                                    // Accumulate pseudo-noise from a few neighborhood samples to form blobs
                                    let p = |r: i32, c: i32| -> u32 {
                                        let rr = (r.max(0) as u32).wrapping_mul(1103515245);
                                        let cc = (c.max(0) as u32).wrapping_mul(12345);
                                        rr ^ cc ^ 0x9E3779B9
                                    };
                                    let r = row as i32;
                                    let c = col as i32;
                                    let n = p(r, c)
                                        .wrapping_add(p(r-1, c+2))
                                        .wrapping_add(p(r+1, c-1))
                                        .wrapping_add(p(r-2, c))
                                        .wrapping_add(p(r, c+3));
                                    // Normalize to [0, 99]
                                    let d = (n % 100) as u32;
                                    // More density near the drink top, fading with depth
                                    let fade = 100u32.saturating_sub(((depth * 100) / (max_depth.max(1))).min(100) as u32);
                                    let density = d.saturating_add(fade / 3);
                                    if density > 140 {
                                        streak_dark
                                    } else if density > 110 {
                                        streak_med
                                    } else if density > 90 {
                                        streak_light
                                    } else {
                                        cup_white
                                    }
                                }
                            };
                            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
                        } else {
                            // Preserve other chars and spacing as-is
                            spans.push(Span::raw(ch.to_string()));
                        }
                    }
                    Line::from(spans)
                })
                .collect();
            let right = Paragraph::new(right_lines)
                .alignment(Alignment::Left)
                .block(Block::default());
            f.render_widget(right, title_row[1]);

            // Menu
            let items: Vec<ListItem> = options
                .iter()
                .enumerate()
                .map(|(i, (label, _))| {
                    let style = if i == selected {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Black)
                    };
                    // Two-line item: label + spacer line for extra vertical padding
                    ListItem::new(vec![
                        Line::from(Span::styled(*label, style)),
                        Line::from("")
                    ])
                })
                .collect();
            let list = List::new(items)
                .block(Block::default())
                .highlight_style(Style::default().bg(Color::Rgb(180, 180, 180)))
                .highlight_symbol("▶ ");
            f.render_widget(list, chunks[1]);

            // Footer
            let footer = Paragraph::new(Line::from(vec![
                Span::styled("↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" to navigate  "),
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" to select  "),
                Span::styled("Q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" to quit"),
            ]))
            .alignment(Alignment::Center)
            .block(Block::default());
            f.render_widget(footer, chunks[2]);
        });

        if let Ok(true) = event::poll(std::time::Duration::from_millis(200)) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break StartAction::Quit,
                        KeyCode::Up => {
                            if selected == 0 { selected = options.len() - 1; } else { selected -= 1; }
                        }
                        KeyCode::Down => {
                            selected = (selected + 1) % options.len();
                        }
                        KeyCode::Enter => break options[selected].1.clone(),
                        _ => {}
                    }
                }
            }
        }
    };

    // Restore terminal
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    result
}


pub fn render_score_breakdown_data(f: &mut Frame, mut score_data: Vec<(String, ScoreBreakdown)>) {
    let area = f.area();

    score_data.sort_by(|a, b| b.1.total_score.partial_cmp(&a.1.total_score).unwrap_or(std::cmp::Ordering::Equal));
    
    let mut rows = Vec::new();
    for (rank, (name, breakdown)) in score_data.iter().enumerate() {
        let rank_cell = Cell::from(format!("#{}", rank + 1));
        let name_cell = Cell::from(Span::styled(name.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        let total_cell = Cell::from(format!("{:.1}", breakdown.total_score));
        
        rows.push(Row::new(vec![rank_cell, name_cell, total_cell]));
        
        // Add breakdown rows
        for category in &breakdown.category_scores {
            let cat_cell = Cell::from(format!("  └─ {}", category.category));
            let pts_cell = Cell::from(format!("{:.1}", category.points));
            rows.push(Row::new(vec![Cell::from(""), cat_cell, pts_cell]));
        }
        
        for bonus in &breakdown.set_bonuses {
            let bonus_cell = Cell::from(format!("  └─ {} (Set Bonus)", bonus.description));
            let pts_cell = Cell::from(format!("{:.1}", bonus.points));
            rows.push(Row::new(vec![Cell::from(""), bonus_cell, pts_cell]));
        }
    }
    
    let widths = [Constraint::Length(4), Constraint::Percentage(60), Constraint::Percentage(40)];
    let table = Table::new(rows, widths)
        .block(Block::default().borders(Borders::ALL).title("Final Scores - Press Q to exit"));
    f.render_widget(table, area);
}

pub fn render_score_breakdown(f: &mut Frame, game: &Game) {
    let players_public = game.get_players_public();
    let mut score_data: Vec<(String, ScoreBreakdown)> = Vec::new();

    for player in &players_public {
        match game.calculate_player_score(player.id) {
            Ok((_total, breakdown)) => {
                score_data.push((player.name.clone(), breakdown));
            }
            Err(_) => {}
        }
    }

    render_score_breakdown_data(f, score_data);
}

pub fn run_local_game() -> Result<(), GameError> {
    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).expect("create terminal");

    // Initialize game with 2 players
    let mut config = GameConfig::default();
    config.player_names = vec!["Player 1".to_string(), "Player 2".to_string()];
    
    let mut game = match Game::new(config) {
        Ok(g) => g,
        Err(e) => {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            return Err(e);
        }
    };

    let mut current_player_id = 0;
    let mut ui_state = GameUIState::new();
    let mut show_scores = false;

    let result = loop {
        let status = game.get_game_status();
        
        if status.is_game_over && show_scores {
            // Show final scores
            let _ = terminal.draw(|f| {
                render_score_breakdown(f, &game);
            });
            
            if let Ok(true) = event::poll(std::time::Duration::from_millis(200)) {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                            _ => {}
                        }
                    }
                }
            }
            continue;
        }

        if status.is_game_over {
            show_scores = true;
            continue;
        }

        // Check if current player has already selected
        let player_state = game.get_player_turn_state(current_player_id).unwrap_or(PlayerTurnState::NotSelected);
        
        if player_state == PlayerTurnState::NotSelected {
            let has_drink_tray_activated = ui_state.drink_tray_activated.get(&current_player_id).copied().unwrap_or(false);
            let max_selections = if has_drink_tray_activated { 2 } else { 1 };

            let _ = terminal.draw(|f| {
                let mut game_view = crate::engine::GamePlayerView::new(&mut game, current_player_id);
                render_game_ui(f, &mut game_view, &ui_state, false, max_selections);
            });

            if let Ok(true) = event::poll(std::time::Duration::from_millis(200)) {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                            KeyCode::Char('h') => {
                                if ui_state.current_view != GameView::Hand {
                                    ui_state.view_history.push(ui_state.current_view);
                                    ui_state.current_view = GameView::Hand;
                                }
                            }
                            KeyCode::Char('m') => {
                                if ui_state.current_view != GameView::MyCards {
                                    ui_state.view_history.push(ui_state.current_view);
                                    ui_state.current_view = GameView::MyCards;
                                    ui_state.my_cards_selection_index = 0;
                                }
                            }
                            KeyCode::Char('p') => {
                                if ui_state.current_view != GameView::PlayerCards {
                                    ui_state.view_history.push(ui_state.current_view);
                                    ui_state.current_view = GameView::PlayerCards;
                                    ui_state.viewing_player_id = ui_state.player_list_index;
                                }
                            }
                            KeyCode::Left => {
                                // Go back in navigation history
                                if let Some(previous_view) = ui_state.view_history.pop() {
                                    ui_state.current_view = previous_view;
                                }
                            }
                            KeyCode::Right => {
                                // Cycle forward through views
                                ui_state.current_view = match ui_state.current_view {
                                    GameView::Hand => {
                                        ui_state.view_history.push(GameView::Hand);
                                        GameView::MyCards
                                    }
                                    GameView::MyCards => {
                                        ui_state.view_history.push(GameView::MyCards);
                                        GameView::PlayerCards
                                    }
                                    GameView::PlayerCards => {
                                        ui_state.view_history.push(GameView::PlayerCards);
                                        GameView::Hand
                                    }
                                };
                                // Reset selection indices when changing views
                                match ui_state.current_view {
                                    GameView::MyCards => ui_state.my_cards_selection_index = 0,
                                    GameView::PlayerCards => ui_state.player_list_index = 0,
                                    _ => {}
                                }
                            }
                            KeyCode::Up => {
                                if ui_state.current_view == GameView::PlayerCards {
                                    if ui_state.player_list_index == 0 {
                                        ui_state.player_list_index = game.num_players() - 1;
                                    } else {
                                        ui_state.player_list_index -= 1;
                                    }
                                    ui_state.viewing_player_id = ui_state.player_list_index;
                                } else if ui_state.current_view == GameView::MyCards {
                                    // Navigate my cards
                                    let player_public = game.get_player_public(current_player_id).unwrap();
                                    let mut card_count = 0;
                                    for (_, count) in &player_public.public_cards {
                                        if *count > 0 {
                                            card_count += 1;
                                        }
                                    }
                                    for (_, count) in &player_public.boosted_fruit_teas {
                                        if *count > 0 {
                                            card_count += 1;
                                        }
                                    }
                                    if card_count > 0 {
                                        if ui_state.my_cards_selection_index == 0 {
                                            ui_state.my_cards_selection_index = card_count - 1;
                                        } else {
                                            ui_state.my_cards_selection_index -= 1;
                                        }
                                    }
                                } else {
                                    if let Ok(hand) = game.get_player_hand(current_player_id) {
                                        let hand_vec: Vec<(CardKind, usize)> = hand.iter()
                                            .filter(|(_, count)| **count > 0)
                                            .map(|(k, v)| (*k, *v))
                                            .collect();
                                        if !hand_vec.is_empty() {
                                            if ui_state.hand_selection_index == 0 {
                                                ui_state.hand_selection_index = hand_vec.len() - 1;
                                            } else {
                                                ui_state.hand_selection_index -= 1;
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if ui_state.current_view == GameView::PlayerCards {
                                    ui_state.player_list_index = (ui_state.player_list_index + 1) % game.num_players();
                                    ui_state.viewing_player_id = ui_state.player_list_index;
                                } else if ui_state.current_view == GameView::MyCards {
                                    // Navigate my cards
                                    let player_public = game.get_player_public(current_player_id).unwrap();
                                    let mut card_count = 0;
                                    for (_, count) in &player_public.public_cards {
                                        if *count > 0 {
                                            card_count += 1;
                                        }
                                    }
                                    for (_, count) in &player_public.boosted_fruit_teas {
                                        if *count > 0 {
                                            card_count += 1;
                                        }
                                    }
                                    if card_count > 0 {
                                        ui_state.my_cards_selection_index = (ui_state.my_cards_selection_index + 1) % card_count;
                                    }
                                } else {
                                    if let Ok(hand) = game.get_player_hand(current_player_id) {
                                        let hand_vec: Vec<(CardKind, usize)> = hand.iter()
                                            .filter(|(_, count)| **count > 0)
                                            .map(|(k, v)| (*k, *v))
                                            .collect();
                                        if !hand_vec.is_empty() {
                                            ui_state.hand_selection_index = (ui_state.hand_selection_index + 1) % hand_vec.len();
                                        }
                                    }
                                }
                            }
                            KeyCode::Char(' ') => {
                                if ui_state.current_view == GameView::Hand {
                                    if let Ok(hand) = game.get_player_hand(current_player_id) {
                                        let hand_vec: Vec<(CardKind, usize)> = hand.iter()
                                            .filter(|(_, count)| **count > 0)
                                            .map(|(k, v)| (*k, *v))
                                            .collect();

                                    if !hand_vec.is_empty() && ui_state.hand_selection_index < hand_vec.len() {
                                        let (card_kind, available_count) = hand_vec[ui_state.hand_selection_index];

                                        let player_selected = ui_state.player_selections.entry(current_player_id)
                                            .or_insert_with(|| (HashMap::new(), HashMap::new()));
                                        let already_selected = player_selected.0.get(&card_kind).copied().unwrap_or(0);

                                        // Recalculate current selections from the actual selection state
                                        let current_selections_count: usize = player_selected.0.values().sum();

                                        if already_selected > 0 {
                                            // Check if we can select more of this card type
                                            let can_select_more = current_selections_count < max_selections;
                                            let has_available = already_selected < available_count;

                                            if can_select_more && has_available {
                                                // Select more copies of this card type
                                                let to_select = (max_selections - current_selections_count).min(available_count - already_selected);
                                                *player_selected.0.entry(card_kind).or_insert(0) += to_select;
                                            } else {
                                                // Unselect: decrease count
                                                if let Some(count) = player_selected.0.get_mut(&card_kind) {
                                                    *count -= 1;
                                                    if *count == 0 {
                                                        player_selected.0.remove(&card_kind);
                                                    }
                                                }
                                            }
                                        } else if current_selections_count < max_selections {
                                            // Select: increase count (can select multiple copies if Drink Tray is active)
                                            let to_select = (max_selections - current_selections_count).min(available_count);
                                            *player_selected.0.entry(card_kind).or_insert(0) += to_select;
                                        }
                                    }
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                // Use Drink Tray from My Cards view (move from public_cards to hand)
                                if ui_state.current_view == GameView::MyCards {
                                    let player_public = game.get_player_public(current_player_id).unwrap();
                                    let mut card_list: Vec<(CardKind, usize, bool)> = Vec::new();
                                    for (card_kind, count) in &player_public.public_cards {
                                        if *count > 0 {
                                            card_list.push((*card_kind, *count, false));
                                        }
                                    }
                                    for (card_kind, count) in &player_public.boosted_fruit_teas {
                                        if *count > 0 {
                                            card_list.push((*card_kind, *count, true));
                                        }
                                    }

                                    if !card_list.is_empty() && ui_state.my_cards_selection_index < card_list.len() {
                                        let (card_kind, _, is_boosted) = card_list[ui_state.my_cards_selection_index];
                                        if card_kind == CardKind::DrinkTray && !is_boosted {
                                            if game.activate_drink_tray(current_player_id).is_ok() {
                                                ui_state.drink_tray_activated.insert(current_player_id, true);
                                                ui_state.view_history.push(GameView::MyCards);
                                                ui_state.current_view = GameView::Hand;
                                                ui_state.hand_selection_index = 0;
                                            }
                                        }
                                    }
                                } else if ui_state.current_view == GameView::Hand {
                                    // Confirm selection - only works in Hand view
                                    let player_selected = ui_state.player_selections.entry(current_player_id)
                                        .or_insert_with(|| (HashMap::new(), HashMap::new()));
                                    let total_selected: usize = player_selected.0.values().sum();

                                    if total_selected >= max_selections {
                                        // Get fresh hand (in case Drink Tray was activated this turn)
                                        let current_hand_result = game.get_player_hand(current_player_id);
                                        let current_hand = match current_hand_result {
                                            Ok(h) => h,
                                            Err(_) => {
                                                player_selected.0.clear();
                                                continue;
                                            }
                                        };

                                        // Build remaining hand (current_hand minus selected cards)
                                        let mut remaining_hand = current_hand.clone();
                                        for (kind, count) in &player_selected.0 {
                                            if let Some(remaining_count) = remaining_hand.get_mut(kind) {
                                                *remaining_count -= count;
                                                if *remaining_count == 0 {
                                                    remaining_hand.remove(kind);
                                                }
                                            }
                                        }

                                        // Validate submission
                                        if let Err(_) = game.validate_hand_submission(current_player_id, &player_selected.0, &remaining_hand) {
                                            // Reset selection on validation error
                                            player_selected.0.clear();
                                        } else {
                                            // Store remaining hand
                                            player_selected.1 = remaining_hand;

                                            // Mark player as selected
                                            if let Err(_) = game.mark_player_selected(current_player_id) {
                                                break Err(GameError::InvalidConfig);
                                            }

                                            // Reset Drink Tray activation for this player
                                            ui_state.drink_tray_activated.remove(&current_player_id);

                                            // Move to next player
                                            current_player_id = (current_player_id + 1) % game.num_players();
                                            ui_state.hand_selection_index = 0;
                                        }
                                    }
                                }
                            }
                            KeyCode::Backspace | KeyCode::Char('r') => {
                                // Reset selection for current player
                                if ui_state.current_view == GameView::Hand {
                                    if let Some(player_selected) = ui_state.player_selections.get_mut(&current_player_id) {
                                        player_selected.0.clear();
                                    }
                                }
                            }
                            KeyCode::Char('u') => {
                                // Unuse Drink Tray (move from hand back to public_cards)
                                if ui_state.current_view == GameView::Hand && has_drink_tray_activated {
                                    let player = &mut game.players[current_player_id];
                                    if let Some(drink_tray_count) = player.hand.get_mut(&CardKind::DrinkTray) {
                                        if *drink_tray_count > 0 {
                                            *drink_tray_count -= 1;
                                            if *drink_tray_count == 0 {
                                                player.hand.remove(&CardKind::DrinkTray);
                                            }
                                            // Move back to public_cards
                                            *player.public_cards.entry(CardKind::DrinkTray).or_insert(0) += 1;
                                            // Deactivate Drink Tray
                                            ui_state.drink_tray_activated.remove(&current_player_id);
                                            // Navigate to My Cards view
                                            ui_state.view_history.push(GameView::Hand);
                                            ui_state.current_view = GameView::MyCards;
                                            ui_state.my_cards_selection_index = 0;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else {
            // All players have selected, process turn
            if game.all_players_selected() {
                // Build submissions for all players
                let mut submissions = Vec::new();
                for player_id in 0..game.num_players() {
                    if let Some((selected, remaining)) = ui_state.player_selections.get(&player_id) {
                        submissions.push(Some((selected.clone(), remaining.clone())));
                    } else {
                        // Player didn't have a selection stored, skip them
                        submissions.push(None);
                    }
                }

                // Process the turn (this handles passing hands, on_draft actions, and calling next_turn/start_new_round)
                if let Err(e) = game.process_turn(submissions) {
                    break Err(e);
                }

                // Clear selections
                ui_state.player_selections.clear();
                current_player_id = 0;
                ui_state.hand_selection_index = 0;
            } else {
                // Wait for other players (in local game, rotate through players)
                current_player_id = (current_player_id + 1) % game.num_players();
            }
        }
    };

    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    result
}


