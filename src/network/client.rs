use libp2p::{
    gossipsub::IdentTopic,
    swarm::Swarm,
    Multiaddr,
};
use std::error::Error;

use crate::network::behaviour::BobaGoBehaviour;

/// Generic client with networking (works for any state type)
pub struct Client<S> {
    pub(crate) swarm: Swarm<BobaGoBehaviour>,
    pub(crate) state: S,
    pub(crate) topic: IdentTopic,
}

// Generic impl - works for any state type
impl<S> Client<S> {
    /// Connect to a host at the given address
    pub fn connect(&mut self, addr: &str) -> Result<(), Box<dyn Error>> {
        let remote_addr: Multiaddr = addr.parse()?;
        self.swarm.dial(remote_addr)?;
        Ok(())
    }

    /// Get reference to state
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Get mutable reference to state
    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }
}
