use libp2p::{
    futures::StreamExt,
    gossipsub::IdentTopic,
    swarm::{Swarm, SwarmEvent},
    Multiaddr, PeerId,
};
use std::collections::HashMap;

use crate::network::behaviour::{BobaGoBehaviour, BobaGoBehaviourEvent};
use crate::network::Host;
use crate::engine::{Game, models::CardKind};
use super::state::GameHostState;
use super::protocol::{GameClientMessage, GameHostMessage, GameEndReason};
use crate::log;

impl Host<GameHostState> {
    pub fn new(
        swarm: Swarm<BobaGoBehaviour>,
        topic: IdentTopic,
        game: Game,
        peer_to_player_id: HashMap<PeerId, usize>,
        player_id_to_peer: HashMap<usize, PeerId>,
    ) -> Self {
        let state = GameHostState::new(game, peer_to_player_id, player_id_to_peer);
        Self {
            swarm,
            state,
            topic,
        }
    }
    // process turn submission from a player
    fn process_turn_submission(
        &mut self,
        peer: PeerId,
        selected_cards: HashMap<CardKind, usize>,
        remaining_hand: HashMap<CardKind, usize>,
    ) -> (GameHostMessage, Option<GameHostEvent>) {
        let player_id = match self.state.get_player_id(&peer) {
            Some(id) => id,
            None => {
                return (
                    GameHostMessage::Error {
                        message: "Player not found".to_string(),
                    },
                    None,
                );
            }
        };

        log::host(format!("Turn submission from player {player_id}"));

        // validate submission
        if let Err(e) = self.state.game.validate_hand_submission(
            player_id,
            &selected_cards,
            &remaining_hand,
        ) {
            return (
                GameHostMessage::Error {
                    message: format!("Invalid submission: {:?}", e),
                },
                None,
            );
        }

        // mark player as selected
        if let Err(e) = self.state.game.mark_player_selected(player_id) {
            return (
                GameHostMessage::Error {
                    message: format!("Failed to mark player: {:?}", e),
                },
                None,
            );
        }

        // check if all players have submitted
        if self.state.game.all_players_selected() {
            return (
                GameHostMessage::Error {
                    message: "Submission accepted".to_string(),
                },
                Some(GameHostEvent::AllPlayersSubmitted),
            );
        }

        (
            GameHostMessage::Error {
                message: "Submission accepted".to_string(),
            },
            Some(GameHostEvent::PlayerSubmitted { player_id }),
        )
    }

    // broadcast game update to all players
    fn broadcast_game_update(&mut self) {
        let players_public = self.state.game.get_players_public();
        let game_status = self.state.game.get_game_status();

        for (peer_id, player_id) in &self.state.peer_to_player_id {
            if let Ok(hand) = self.state.game.get_player_hand(*player_id) {
                let message = GameHostMessage::GameUpdate {
                    your_hand: hand.clone(),
                    players_public: players_public.clone(),
                    game_status: game_status.clone(),
                };

                if let Ok(json) = serde_json::to_string(&message) {
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.topic.clone(), json.as_bytes())
                        .ok();
                }
            }
        }
    }

    // broadcast game ended
    fn broadcast_game_ended(&mut self, reason: GameEndReason) {
        let mut final_scores = Vec::new();

        for player_id in 0..self.state.game.num_players() {
            if let Ok((score, _)) = self.state.game.calculate_player_score(player_id) {
                final_scores.push((player_id, score));
            }
        }

        let message = GameHostMessage::GameEnded {
            final_scores,
            reason,
        };

        if let Ok(json) = serde_json::to_string(&message) {
            self.swarm
                .behaviour_mut()
                .gossipsub
                .publish(self.topic.clone(), json.as_bytes())
                .ok();
        }
    }

    // handle request-response network events
    fn handle_request_response(
        &mut self,
        rr_event: libp2p::request_response::Event<GameClientMessage, GameHostMessage>,
    ) -> Option<GameHostEvent> {
        use libp2p::request_response;

        match rr_event {
            request_response::Event::Message { peer, message, .. } => {
                if let request_response::Message::Request {
                    request: GameClientMessage::SubmitTurn { selected_cards, remaining_hand },
                    channel,
                    ..
                } = message
                {
                    let (response, event) = self.process_turn_submission(peer, selected_cards, remaining_hand);

                    self.swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, response)
                        .ok();

                    return event;
                }
            }
            _ => {}
        }
        None
    }

    // handle connection closed event
    fn handle_connection_closed(&mut self, peer_id: PeerId) -> Option<GameHostEvent> {
        log::host(format!("Connection closed with {peer_id}"));
        if let Some(player_id) = self.state.remove_player(&peer_id) {
            self.broadcast_game_ended(GameEndReason::PlayerDisconnected { player_id });
            return Some(GameHostEvent::PlayerDisconnected { peer_id, player_id });
        }
        None
    }

    // run event loop
    pub async fn next_event(&mut self) -> Option<GameHostEvent> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(BobaGoBehaviourEvent::RequestResponse(rr_event)) => {
                    if let Some(event) = self.handle_request_response(rr_event) {
                        return Some(event);
                    }
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    log::host(format!("Connection established with {peer_id}"));
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    if let Some(event) = self.handle_connection_closed(peer_id) {
                        return Some(event);
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
pub enum GameHostEvent {
    PlayerSubmitted { player_id: usize },
    AllPlayersSubmitted,
    PlayerDisconnected { peer_id: PeerId, player_id: usize },
}
