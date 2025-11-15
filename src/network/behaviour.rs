use libp2p::{
    gossipsub, request_response,
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use serde::{Deserialize, Serialize};
use std::io;

use super::lobby::protocol::{ClientMessage, HostMessage};
use super::game::protocol::{GameClientMessage, GameHostMessage};

// unified message enums for lobby + game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRequest {
    Lobby(ClientMessage),
    Game(GameClientMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostResponse {
    Lobby(HostMessage),
    Game(GameHostMessage),
}

#[derive(NetworkBehaviour)]
pub struct BobaGoBehaviour {
    pub request_response: request_response::cbor::Behaviour<ClientRequest, HostResponse>,
    pub gossipsub: gossipsub::Behaviour,
}

impl BobaGoBehaviour {
    pub fn new(_peer_id: libp2p::PeerId) -> Result<Self, io::Error> {
        let request_response = request_response::cbor::Behaviour::new(
            [(StreamProtocol::new("/boba-go/lobby/1.0.0"), request_response::ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        // config: 1s heartbeat, strict validation
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // create gossipsub with signed messages
        let gossipsub: gossipsub::Behaviour = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(libp2p::identity::Keypair::generate_ed25519()),
            gossipsub_config,
        ).map_err(|e: &'static str| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(Self {
            request_response,
            gossipsub,
        })
    }
}
