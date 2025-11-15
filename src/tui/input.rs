use crossterm::event::KeyCode;
use std::collections::HashMap;
use crate::engine::CardKind;
use crate::tui::{GameView, GameUIState, GameInterface};

pub enum InputAction {
    Quit,
    SubmitTurn,
    Continue,
}

pub fn calculate_max_selections(ui_state: &GameUIState, player_id: usize) -> usize {
    if ui_state.drink_tray_activated.get(&player_id).copied().unwrap_or(false) {
        2
    } else {
        1
    }
}

pub fn handle_game_input<G: GameInterface>(
    key_code: KeyCode,
    game: &mut G,
    ui_state: &mut GameUIState,
    max_selections: usize,
) -> InputAction {
    let player_id = game.get_player_id();

    match key_code {
        KeyCode::Char('q') | KeyCode::Esc => InputAction::Quit,
        KeyCode::Char('h') => {
            if ui_state.current_view != GameView::Hand {
                ui_state.view_history.push(ui_state.current_view);
                ui_state.current_view = GameView::Hand;
            }
            InputAction::Continue
        }
        KeyCode::Char('m') => {
            if ui_state.current_view != GameView::MyCards {
                ui_state.view_history.push(ui_state.current_view);
                ui_state.current_view = GameView::MyCards;
                ui_state.my_cards_selection_index = 0;
            }
            InputAction::Continue
        }
        KeyCode::Char('p') => {
            if ui_state.current_view != GameView::PlayerCards {
                ui_state.view_history.push(ui_state.current_view);
                ui_state.current_view = GameView::PlayerCards;
                ui_state.viewing_player_id = ui_state.player_list_index;
            }
            InputAction::Continue
        }
        KeyCode::Left => {
            if let Some(previous_view) = ui_state.view_history.pop() {
                ui_state.current_view = previous_view;
            }
            InputAction::Continue
        }
        KeyCode::Right => {
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
            match ui_state.current_view {
                GameView::MyCards => ui_state.my_cards_selection_index = 0,
                GameView::PlayerCards => ui_state.player_list_index = 0,
                _ => {}
            }
            InputAction::Continue
        }
        KeyCode::Up => {
            handle_up_navigation(game, ui_state);
            InputAction::Continue
        }
        KeyCode::Down => {
            handle_down_navigation(game, ui_state);
            InputAction::Continue
        }
        KeyCode::Char(' ') => {
            handle_card_selection(game, ui_state, max_selections);
            InputAction::Continue
        }
        KeyCode::Enter => {
            handle_enter_key(game, ui_state, max_selections)
        }
        KeyCode::Backspace | KeyCode::Char('r') => {
            if ui_state.current_view == GameView::Hand {
                if let Some(player_selected) = ui_state.player_selections.get_mut(&player_id) {
                    player_selected.0.clear();
                }
            }
            InputAction::Continue
        }
        KeyCode::Char('u') => {
            handle_undo_drink_tray(game, ui_state);
            InputAction::Continue
        }
        _ => InputAction::Continue,
    }
}

fn handle_up_navigation<G: GameInterface>(game: &G, ui_state: &mut GameUIState) {
    match ui_state.current_view {
        GameView::PlayerCards => {
            let num_players = game.get_players_public().len();
            if ui_state.player_list_index == 0 {
                ui_state.player_list_index = num_players - 1;
            } else {
                ui_state.player_list_index -= 1;
            }
            ui_state.viewing_player_id = ui_state.player_list_index;
        }
        GameView::MyCards => {
            let player_id = game.get_player_id();
            let players_public = game.get_players_public();
            if let Some(player_public) = players_public.get(player_id) {
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
            }
        }
        GameView::Hand => {
            let hand = game.get_hand();
            let hand_vec: Vec<_> = hand.iter().filter(|(_, c)| **c > 0).collect();
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

fn handle_down_navigation<G: GameInterface>(game: &G, ui_state: &mut GameUIState) {
    match ui_state.current_view {
        GameView::PlayerCards => {
            let num_players = game.get_players_public().len();
            ui_state.player_list_index = (ui_state.player_list_index + 1) % num_players;
            ui_state.viewing_player_id = ui_state.player_list_index;
        }
        GameView::MyCards => {
            let player_id = game.get_player_id();
            let players_public = game.get_players_public();
            if let Some(player_public) = players_public.get(player_id) {
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
            }
        }
        GameView::Hand => {
            let hand = game.get_hand();
            let hand_vec: Vec<_> = hand.iter().filter(|(_, c)| **c > 0).collect();
            if !hand_vec.is_empty() {
                ui_state.hand_selection_index = (ui_state.hand_selection_index + 1) % hand_vec.len();
            }
        }
    }
}

fn handle_card_selection<G: GameInterface>(game: &G, ui_state: &mut GameUIState, max_selections: usize) {
    if ui_state.current_view != GameView::Hand {
        return;
    }

    let hand = game.get_hand();
    let hand_vec: Vec<(CardKind, usize)> = hand.iter()
        .filter(|(_, c)| **c > 0)
        .map(|(k, v)| (*k, *v))
        .collect();

    if hand_vec.is_empty() || ui_state.hand_selection_index >= hand_vec.len() {
        return;
    }

    let (card_kind, available_count) = hand_vec[ui_state.hand_selection_index];
    let player_id = game.get_player_id();
    let player_selected = ui_state.player_selections
        .entry(player_id)
        .or_insert_with(|| (HashMap::new(), HashMap::new()));
    let already_selected = player_selected.0.get(&card_kind).copied().unwrap_or(0);
    let current_selections_count: usize = player_selected.0.values().sum();

    if already_selected > 0 {
        let can_select_more = current_selections_count < max_selections;
        let has_available = already_selected < available_count;

        if can_select_more && has_available {
            let to_select = (max_selections - current_selections_count).min(available_count - already_selected);
            *player_selected.0.entry(card_kind).or_insert(0) += to_select;
        } else {
            if let Some(count) = player_selected.0.get_mut(&card_kind) {
                *count -= 1;
                if *count == 0 {
                    player_selected.0.remove(&card_kind);
                }
            }
        }
    } else if current_selections_count < max_selections {
        let to_select = (max_selections - current_selections_count).min(available_count);
        *player_selected.0.entry(card_kind).or_insert(0) += to_select;
    }
}

fn handle_enter_key<G: GameInterface>(
    game: &mut G,
    ui_state: &mut GameUIState,
    max_selections: usize,
) -> InputAction {
    let player_id = game.get_player_id();

    match ui_state.current_view {
        GameView::MyCards => {
            handle_drink_tray_activation(game, ui_state);
            InputAction::Continue
        }
        GameView::Hand => {
            if let Some((selected, _)) = ui_state.player_selections.get(&player_id) {
                let total_selected: usize = selected.values().sum();
                if total_selected >= max_selections && !selected.is_empty() {
                    let hand = game.get_hand();
                    let mut remaining = hand.clone();
                    for (kind, count) in selected {
                        if let Some(remaining_count) = remaining.get_mut(kind) {
                            *remaining_count = remaining_count.saturating_sub(*count);
                        }
                    }

                    if game.submit_turn(selected.clone(), remaining).is_ok() {
                        return InputAction::SubmitTurn;
                    }
                }
            }
            InputAction::Continue
        }
        _ => InputAction::Continue,
    }
}

fn handle_drink_tray_activation<G: GameInterface>(game: &mut G, ui_state: &mut GameUIState) {
    let player_id = game.get_player_id();
    let players_public = game.get_players_public();

    if let Some(player_public) = players_public.get(player_id) {
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
                ui_state.drink_tray_activated.insert(player_id, true);
                ui_state.view_history.push(GameView::MyCards);
                ui_state.current_view = GameView::Hand;
                ui_state.hand_selection_index = 0;
            }
        }
    }
}

fn handle_undo_drink_tray<G: GameInterface>(game: &G, ui_state: &mut GameUIState) {
    let player_id = game.get_player_id();
    if ui_state.current_view == GameView::Hand
        && ui_state.drink_tray_activated.get(&player_id).copied().unwrap_or(false)
    {
        ui_state.drink_tray_activated.remove(&player_id);
        ui_state.view_history.push(GameView::Hand);
        ui_state.current_view = GameView::MyCards;
        ui_state.my_cards_selection_index = 0;
    }
}
