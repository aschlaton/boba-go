use std::collections::HashMap;
use libp2p::{
    futures::StreamExt,
    gossipsub::IdentTopic,
    swarm::{Swarm, SwarmEvent},
    PeerId,
};

use crate::network::behaviour::{BobaGoBehaviour, BobaGoBehaviourEvent, ClientRequest};
use crate::network::Client;
use crate::engine::{models::{CardKind, PlayerPublic}, state::{GameStatus, PlayerTurnState}};
use super::protocol::{GameClientMessage, GameHostMessage, GameEndReason};
use crate::log;

pub struct GameClientState {
    pub player_id: usize,
    pub hand: HashMap<CardKind, usize>,
    pub players_public: Vec<PlayerPublic>,
    pub game_status: GameStatus,
    pub selected_cards: HashMap<CardKind, usize>,
    pub turn_submitted: bool,
    pub host_peer_id: Option<PeerId>,
}

impl GameClientState {
    pub fn new(
        player_id: usize,
        hand: HashMap<CardKind, usize>,
        players_public: Vec<PlayerPublic>,
        game_status: GameStatus,
        host_peer_id: Option<PeerId>,
    ) -> Self {
        Self {
            player_id,
            hand,
            players_public,
            game_status,
            selected_cards: HashMap::new(),
            turn_submitted: false,
            host_peer_id,
        }
    }

    pub fn select_card(&mut self, card: CardKind) -> Result<(), String> {
        let hand_count = self.hand.get(&card).copied().unwrap_or(0);
        let selected_count = self.selected_cards.get(&card).copied().unwrap_or(0);

        if selected_count >= hand_count {
            return Err("Cannot select more cards than in hand".to_string());
        }

        *self.selected_cards.entry(card).or_insert(0) += 1;
        Ok(())
    }

    pub fn deselect_card(&mut self, card: CardKind) -> Result<(), String> {
        let selected_count = self.selected_cards.get(&card).copied().unwrap_or(0);

        if selected_count == 0 {
            return Err("No cards of this type selected".to_string());
        }

        if selected_count == 1 {
            self.selected_cards.remove(&card);
        } else {
            *self.selected_cards.get_mut(&card).unwrap() -= 1;
        }

        Ok(())
    }

    pub fn clear_selection(&mut self) {
        self.selected_cards.clear();
    }

    // get remaining hand after removing selected cards
    pub fn get_remaining_hand(&self) -> HashMap<CardKind, usize> {
        let mut remaining = self.hand.clone();

        for (card, selected_count) in &self.selected_cards {
            if let Some(hand_count) = remaining.get_mut(card) {
                *hand_count = hand_count.saturating_sub(*selected_count);
                if *hand_count == 0 {
                    remaining.remove(card);
                }
            }
        }

        remaining
    }

    pub fn mark_turn_submitted(&mut self) {
        self.turn_submitted = true;
    }

    pub fn reset_for_new_turn(&mut self) {
        self.selected_cards.clear();
        self.turn_submitted = false;
    }

    pub fn update_hand(&mut self, new_hand: HashMap<CardKind, usize>) {
        self.hand = new_hand;
        self.reset_for_new_turn();
    }

    pub fn update_game_status(&mut self, status: GameStatus) {
        self.game_status = status;
    }

    pub fn update_players_public(&mut self, players: Vec<PlayerPublic>) {
        self.players_public = players;
    }

    pub fn can_submit(&self) -> bool {
        !self.selected_cards.is_empty() && !self.turn_submitted
    }

    pub fn get_own_turn_state(&self) -> PlayerTurnState {
        self.game_status.player_turn_states
            .get(self.player_id)
            .copied()
            .unwrap_or(PlayerTurnState::NotSelected)
    }
}

impl Client<GameClientState> {
    pub fn new(
        swarm: Swarm<BobaGoBehaviour>,
        topic: IdentTopic,
        player_id: usize,
        hand: HashMap<CardKind, usize>,
        players_public: Vec<PlayerPublic>,
        game_status: GameStatus,
        host_peer_id: Option<PeerId>,
    ) -> Self {
        let state = GameClientState::new(player_id, hand, players_public, game_status, host_peer_id);
        Self {
            swarm,
            state,
            topic,
        }
    }

    // submit turn to host
    pub fn submit_turn(&mut self, host_peer: PeerId) {
        let message = ClientRequest::Game(GameClientMessage::SubmitTurn {
            selected_cards: self.state.selected_cards.clone(),
            remaining_hand: self.state.get_remaining_hand(),
        });

        self.swarm
            .behaviour_mut()
            .request_response
            .send_request(&host_peer, message);

        self.state.mark_turn_submitted();
        log::client(format!("Submitted turn to host"));
    }

    // run event loop
    pub async fn next_event(&mut self) -> Option<GameClientEvent> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(BobaGoBehaviourEvent::Gossipsub(gossipsub_event)) => {
                    if let libp2p::gossipsub::Event::Message { message, .. } = gossipsub_event {
                        if let Ok(json_str) = std::str::from_utf8(&message.data) {
                            if let Ok(host_message) = serde_json::from_str::<GameHostMessage>(json_str) {
                                match host_message {
                                    GameHostMessage::GameUpdate { all_hands, players_public, game_status } => {
                                        // extract own hand from all_hands
                                        if let Some(your_hand) = all_hands.get(self.state.player_id) {
                                            self.state.update_hand(your_hand.clone());
                                        }
                                        self.state.update_players_public(players_public);
                                        self.state.update_game_status(game_status.clone());
                                        return Some(GameClientEvent::GameUpdated { game_status });
                                    }
                                    GameHostMessage::GameEnded { final_scores, reason } => {
                                        return Some(GameClientEvent::GameEnded { final_scores, reason });
                                    }
                                    GameHostMessage::Error { message } => {
                                        log::client(format!("Error from host: {}", message));
                                    }
                                }
                            }
                        }
                    }
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    super::super::events::handle_client_connection_established(&mut self.swarm, peer_id);
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    super::super::events::log_client_connection_closed(peer_id);
                    if Some(peer_id) == self.state.host_peer_id {
                        return Some(GameClientEvent::Disconnected);
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
pub enum GameClientEvent {
    GameUpdated { game_status: GameStatus },
    GameEnded { final_scores: Vec<(usize, f32, String, crate::engine::ScoreBreakdown)>, reason: GameEndReason },
    Disconnected,
}

impl crate::tui::GameInterface for Client<GameClientState> {
    fn get_hand(&self) -> HashMap<CardKind, usize> {
        self.state.hand.clone()
    }

    fn get_game_status(&self) -> crate::engine::state::GameStatus {
        self.state.game_status.clone()
    }

    fn get_players_public(&self) -> Vec<crate::engine::models::PlayerPublic> {
        self.state.players_public.clone()
    }

    fn submit_turn(&mut self, selected: HashMap<CardKind, usize>, _remaining: HashMap<CardKind, usize>) -> Result<(), String> {
        if let Some(host_peer) = self.state.host_peer_id {
            self.state.selected_cards = selected;
            Client::<GameClientState>::submit_turn(self, host_peer);
            Ok(())
        } else {
            Err("No host connected".to_string())
        }
    }

    fn get_player_id(&self) -> usize {
        self.state.player_id
    }

    fn activate_drink_tray(&mut self) -> Result<(), String> {
        if let Some(host_peer) = self.state.host_peer_id {
            let message = ClientRequest::Game(GameClientMessage::ActivateDrinkTray);
            self.swarm
                .behaviour_mut()
                .request_response
                .send_request(&host_peer, message);
            log::client("Sent DrinkTray activation request to host".to_string());
            Ok(())
        } else {
            Err("No host connected".to_string())
        }
    }
}

