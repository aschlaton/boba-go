use libp2p::{PeerId, Swarm};
use crate::network::behaviour::BobaGoBehaviour;

/// Helper function to handle ConnectionEstablished events for hosts
/// Logs the connection and adds the peer as an explicit gossipsub peer
pub fn handle_host_connection_established(
    swarm: &mut Swarm<BobaGoBehaviour>,
    peer_id: PeerId,
) {
    crate::log::host(format!("Connection established with {peer_id}"));
    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
}

/// Helper function to handle ConnectionEstablished events for clients
/// Logs the connection and adds the peer as an explicit gossipsub peer
pub fn handle_client_connection_established(
    swarm: &mut Swarm<BobaGoBehaviour>,
    peer_id: PeerId,
) {
    crate::log::client(format!("Connected to {peer_id}"));
    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
}

/// Helper function to log connection closed for hosts
pub fn log_host_connection_closed(peer_id: PeerId) {
    crate::log::host(format!("Connection closed with {peer_id}"));
}

/// Helper function to log connection closed for clients
pub fn log_client_connection_closed(peer_id: PeerId) {
    crate::log::client(format!("Disconnected from {peer_id}"));
}
