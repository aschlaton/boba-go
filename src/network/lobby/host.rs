use libp2p::{
    core::upgrade,
    futures::StreamExt,
    gossipsub::IdentTopic,
    identity, noise,
    swarm::{Swarm, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Transport,
};
use std::error::Error;

use crate::network::behaviour::{BobaGoBehaviour, BobaGoBehaviourEvent, ClientRequest, HostResponse};
use crate::network::Host;
use super::protocol::{ClientMessage, HostMessage, LobbyPlayer};
use super::state::LobbyHostState;
use crate::log;

// Lobby-specific impl
impl Host<LobbyHostState> {
    pub async fn new(room_name: String, host_player_name: String) -> Result<Self, Box<dyn Error>> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        log::host(format!("Local peer ID: {local_peer_id}"));

        // encrypted, multiplexed TCP transport
        let transport = tcp::tokio::Transport::default()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        let behaviour = BobaGoBehaviour::new(local_peer_id)?;
        let mut swarm = Swarm::new(
            transport,
            behaviour,
            local_peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );

        let topic = IdentTopic::new(format!("boba-go-lobby-{}", room_name));
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        let state = LobbyHostState::new(room_name, host_player_name);

        Ok(Self {
            swarm,
            state,
            topic,
        })
    }

    pub fn get_lobby_players(&self) -> Vec<LobbyPlayer> {
        self.state.get_all_players()
    }

    // process join request, return response and event
    fn process_join_request(&mut self, peer: PeerId, player_name: String) -> (HostMessage, Option<HostEvent>) {
        log::host(format!("Join request from peer {peer} with name '{player_name}'"));

        if self.state.is_name_taken(&player_name) {
            log::host("Name taken: true");
            let response = HostMessage::JoinResponse {
                accepted: false,
                player_id: None,
                rejection_reason: Some("Name already taken".to_string()),
                lobby_players: vec![],
            };
            return (response, None);
        }

        log::host("Name taken: false");

        // add player and broadcast update
        let player_id = self.state.add_player(peer, player_name.clone());
        let lobby_players = self.get_lobby_players();
        self.broadcast_lobby_update();

        let response = HostMessage::JoinResponse {
            accepted: true,
            player_id: Some(player_id),
            rejection_reason: None,
            lobby_players,
        };

        let event = HostEvent::PlayerJoined {
            peer_id: peer,
            player_id,
            player_name,
        };

        (response, Some(event))
    }

    // broadcast lobby update to all clients
    fn broadcast_lobby_update(&mut self) {
        let players = self.get_lobby_players();
        let message = HostMessage::LobbyUpdate { players };

        if let Ok(json) = serde_json::to_string(&message) {
            self.swarm
                .behaviour_mut()
                .gossipsub
                .publish(self.topic.clone(), json.as_bytes())
                .ok();
        }
    }

    /// Handle request-response network events
    fn handle_request_response(&mut self, rr_event: libp2p::request_response::Event<ClientRequest, HostResponse>) -> Option<HostEvent> {
        use libp2p::request_response;

        match rr_event {
            request_response::Event::Message { peer, message, .. } => {
                if let request_response::Message::Request {
                    request: ClientRequest::Lobby(ClientMessage::JoinRequest { player_name }),
                    channel,
                    ..
                } = message
                {
                    let (response, event) = self.process_join_request(peer, player_name);

                    self.swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, HostResponse::Lobby(response))
                        .ok();

                    return event;
                }
            }
            _ => {}
        }
        None
    }

    /// Handle connection closed event
    fn handle_connection_closed(&mut self, peer_id: PeerId, cause: &Option<libp2p::swarm::ConnectionError>) -> Option<HostEvent> {
        log::host(format!("Connection closed with {peer_id}: {cause:?}"));
        if self.state.remove_player(&peer_id).is_some() {
            self.broadcast_lobby_update();
            return Some(HostEvent::PlayerLeft { peer_id });
        }
        None
    }

    /// run event loop
    /// events = listening, join request, player joined/left, gossipsub message, connection established/closed, error
    pub async fn next_event(&mut self) -> Option<HostEvent> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    return Some(HostEvent::Listening { address });
                }
                SwarmEvent::Behaviour(BobaGoBehaviourEvent::RequestResponse(rr_event)) => {
                    if let Some(event) = self.handle_request_response(rr_event) {
                        return Some(event);
                    }
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    log::host(format!("Connection established with {peer_id}"));
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    if let Some(event) = self.handle_connection_closed(peer_id, &cause) {
                        return Some(event);
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
pub enum HostEvent {
    Listening { address: Multiaddr },
    PlayerJoined { peer_id: PeerId, player_id: usize, player_name: String },
    PlayerLeft { peer_id: PeerId },
}
