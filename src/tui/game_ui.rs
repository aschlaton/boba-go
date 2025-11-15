use std::collections::HashMap;
use crate::engine::{CardKind, state::GameStatus, models::PlayerPublic};

#[derive(Copy, Clone, PartialEq)]
pub enum GameView {
    Hand,
    MyCards,
    PlayerCards,
}

pub struct GameUIState {
    pub hand_selection_index: usize,
    pub my_cards_selection_index: usize,
    pub player_selections: HashMap<usize, (HashMap<CardKind, usize>, HashMap<CardKind, usize>)>,
    pub drink_tray_activated: HashMap<usize, bool>,
    pub current_view: GameView,
    pub viewing_player_id: usize,
    pub player_list_index: usize,
    pub view_history: Vec<GameView>,
}

impl GameUIState {
    pub fn new() -> Self {
        Self {
            hand_selection_index: 0,
            my_cards_selection_index: 0,
            player_selections: HashMap::new(),
            drink_tray_activated: HashMap::new(),
            current_view: GameView::Hand,
            viewing_player_id: 0,
            player_list_index: 0,
            view_history: Vec::new(),
        }
    }

    pub fn reset_for_new_turn(&mut self) {
        self.hand_selection_index = 0;
        self.my_cards_selection_index = 0;
    }

    pub fn clear_selections(&mut self) {
        self.player_selections.clear();
    }
}

pub trait GameInterface {
    fn get_hand(&self) -> HashMap<CardKind, usize>;
    fn get_game_status(&self) -> GameStatus;
    fn get_players_public(&self) -> Vec<PlayerPublic>;
    fn submit_turn(&mut self, selected: HashMap<CardKind, usize>, remaining: HashMap<CardKind, usize>) -> Result<(), String>;
    fn get_player_id(&self) -> usize;
}

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render_game_ui<G: GameInterface>(
    f: &mut Frame,
    game: &G,
    ui_state: &GameUIState,
    submitted: bool,
    max_selections: usize,
) {
    use crate::tui::views::{render_hand, render_my_cards, render_player_cards};
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Status bar
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Footer/controls
        ])
        .split(area);

    // Status bar
    let game_status = game.get_game_status();
    let status_text = format!(
        "Round {}/{} | Turn {} | Passing: {:?}{}",
        game_status.round,
        game_status.round_count,
        game_status.turn,
        game_status.pass_direction,
        if submitted { " [SUBMITTED]" } else { "" }
    );
    let status_para = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Game Status"));
    f.render_widget(status_para, chunks[0]);

    let player_id = game.get_player_id();
    let player_selected = ui_state.player_selections.get(&player_id).map(|(sel, _)| sel);

    match ui_state.current_view {
        GameView::Hand => {
            render_hand(f, game, chunks[1], player_selected, ui_state.hand_selection_index);
        }
        GameView::MyCards => {
            render_my_cards(f, game, chunks[1], ui_state.my_cards_selection_index);
        }
        GameView::PlayerCards => {
            render_player_cards(f, game, ui_state.viewing_player_id, chunks[1], ui_state.player_list_index);
        }
    }

    // Footer with selection info and view controls
    let view_hint = match ui_state.current_view {
        GameView::Hand => "H: Hand  M: My Cards  P: Player Cards",
        GameView::MyCards => "H: Hand  M: My Cards  P: Player Cards",
        GameView::PlayerCards => "H: Hand  M: My Cards  P: Player Cards  ↑/↓: Select Player",
    };

    let has_drink_tray_activated = ui_state.drink_tray_activated.get(&player_id).copied().unwrap_or(false);
    let current_selections: usize = player_selected.map(|s| s.values().sum()).unwrap_or(0);

    let selection_text = if submitted {
        format!("{} | Waiting for other players...", view_hint)
    } else if current_selections == 0 {
        if has_drink_tray_activated {
            format!("{} | Select a card ({} remaining) | U: Undo Drink Tray", view_hint, max_selections)
        } else {
            format!("{} | Select a card ({} remaining)", view_hint, max_selections)
        }
    } else if current_selections < max_selections {
        if has_drink_tray_activated {
            format!("{} | Select {} more card(s) ({} total) | U: Undo Drink Tray", view_hint, max_selections - current_selections, max_selections)
        } else {
            format!("{} | Select {} more card(s) ({} total)", view_hint, max_selections - current_selections, max_selections)
        }
    } else {
        if has_drink_tray_activated {
            format!("{} | Press Enter to confirm selection | U: Undo Drink Tray", view_hint)
        } else {
            format!("{} | Press Enter to confirm selection", view_hint)
        }
    };

    let footer = Paragraph::new(selection_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
