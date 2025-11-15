use libp2p::{
    gossipsub::IdentTopic,
    swarm::Swarm,
    Multiaddr,
};
use std::error::Error;

use crate::network::behaviour::BobaGoBehaviour;

/// Generic host with networking (works for any state type)
pub struct Host<S> {
    pub(crate) swarm: Swarm<BobaGoBehaviour>,
    pub(crate) state: S,
    pub(crate) topic: IdentTopic,
}

// Generic impl - works for any state type
impl<S> Host<S> {
    /// Start listening on a local address
    pub fn listen(&mut self, addr: &str) -> Result<(), Box<dyn Error>> {
        let listen_addr: Multiaddr = addr.parse()?;
        self.swarm.listen_on(listen_addr)?;
        Ok(())
    }

    /// Get the listening addresses
    pub fn listeners(&self) -> Vec<Multiaddr> {
        self.swarm.listeners().cloned().collect()
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    // mutable reference
    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }
}
