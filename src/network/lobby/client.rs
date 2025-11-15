use libp2p::{
    core::upgrade,
    futures::StreamExt,
    gossipsub::IdentTopic,
    identity, noise,
    swarm::{Swarm, SwarmEvent},
    tcp, yamux, PeerId, Transport,
};
use std::error::Error;

use crate::network::behaviour::{BobaGoBehaviour, BobaGoBehaviourEvent, ClientRequest, HostResponse};
use crate::network::Client;
use super::protocol::{ClientMessage, HostMessage, LobbyPlayer};
use crate::log;

/// Lobby-specific client state
pub struct LobbyClientState {
    player_name: String,
    player_id: Option<usize>,
    lobby_players: Vec<LobbyPlayer>,
    host_peer_id: Option<PeerId>,
    join_request_sent: bool,
}

// Lobby-specific impl
impl Client<LobbyClientState> {
    pub async fn new(room_name: String, player_name: String) -> Result<Self, Box<dyn Error>> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        log::client(format!("Local peer ID: {local_peer_id}"));

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

        let state = LobbyClientState {
            player_name,
            player_id: None,
            lobby_players: vec![],
            host_peer_id: None,
            join_request_sent: false,
        };

        Ok(Self {
            swarm,
            state,
            topic,
        })
    }

    /// Send join request to host
    fn send_join_request(&mut self, peer_id: PeerId) {
        let request = ClientRequest::Lobby(ClientMessage::JoinRequest {
            player_name: self.state.player_name.clone(),
        });

        self.swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, request);
    }

    pub fn get_lobby_players(&self) -> Vec<LobbyPlayer> {
        self.state.lobby_players.clone()
    }

    pub fn get_player_id(&self) -> Option<usize> {
        self.state.player_id
    }

    /// run event loop
    /// events = lobby join accept/reject, update, disconnect, error
    pub async fn next_event(&mut self) -> Option<ClientEvent> {
        use libp2p::request_response;

        loop {
            match self.swarm.select_next_some().await {
                // handle join response from host
                SwarmEvent::Behaviour(BobaGoBehaviourEvent::RequestResponse(rr_event)) => {
                    match rr_event {
                        request_response::Event::Message { message, .. } => {
                            if let request_response::Message::Response { response, .. } = message {
                                if let HostResponse::Lobby(lobby_response) = response {
                                    match lobby_response {
                                        HostMessage::JoinResponse {
                                            accepted,
                                            player_id,
                                            rejection_reason,
                                            lobby_players,
                                        } => {
                                            if accepted {
                                                self.state.player_id = player_id;
                                                self.state.lobby_players = lobby_players.clone();
                                                return Some(ClientEvent::JoinedLobby {
                                                    player_id: player_id.unwrap(),
                                                    lobby_players,
                                                });
                                            } else {
                                                return Some(ClientEvent::JoinRejected {
                                                    reason: rejection_reason
                                                        .unwrap_or_else(|| "Unknown reason".to_string()),
                                                });
                                            }
                                        }
                                        HostMessage::LobbyUpdate { .. } => {}
                                    }
                                }
                            }
                        }
                        request_response::Event::OutboundFailure { error, .. } => {
                            return Some(ClientEvent::Error {
                                message: format!("Request failed: {error:?}"),
                            });
                        }
                        _ => {}
                    }
                }
                SwarmEvent::Behaviour(BobaGoBehaviourEvent::Gossipsub(gossipsub_event)) => {
                    if let libp2p::gossipsub::Event::Message { message, .. } = gossipsub_event {
                        if let Ok(json_str) = std::str::from_utf8(&message.data) {
                            if let Ok(host_message) =
                                serde_json::from_str::<HostMessage>(json_str)
                            {
                                if let HostMessage::LobbyUpdate { players } = host_message {
                                    self.state.lobby_players = players.clone();
                                    return Some(ClientEvent::LobbyUpdated { players });
                                }
                            }
                        }
                    }
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    log::client(format!("Connected to host: {peer_id}"));
                    self.state.host_peer_id = Some(peer_id);
                    if !self.state.join_request_sent {
                        self.state.join_request_sent = true;
                        self.send_join_request(peer_id);
                    }
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    if Some(peer_id) == self.state.host_peer_id {
                        return Some(ClientEvent::Disconnected);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Events that can occur in the client lobby
#[derive(Debug)]
pub enum ClientEvent {
    JoinedLobby {
        player_id: usize,
        lobby_players: Vec<LobbyPlayer>,
    },
    JoinRejected {
        reason: String,
    },
    LobbyUpdated {
        players: Vec<LobbyPlayer>,
    },
    Disconnected,
    Error {
        message: String,
    },
}

