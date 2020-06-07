//! This module provides a client registry. It stores a client to provider assignment.
//! The client can lookup his ID and find a provider that provides a service for him.
//! This could be storing incoming messages for a client (receiver).
//!
//! Events:
//!
//! - ClientSpread: If a peer is overloaded and cannot serve all clients, he floods a ClientSpread message. Peers with free capacity request a ClientTransfere from the source of the ClientSpread message.

use libp2p::{
    floodsub::{Floodsub, FloodsubEvent},
    kad::{Kademlia, KademliaEvent, QueryResult},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};
use log;

#[derive(NetworkBehaviour)]
pub struct Registry<TStore> {
    pub floodsub: Floodsub,
    pub kad: Kademlia<TStore>,
}

impl<TStore> NetworkBehaviourEventProcess<FloodsubEvent> for Registry<TStore> {
    // Called when `floodsub` produces an event.
    fn inject_event(&mut self, message: FloodsubEvent) {
        log::trace!("new floodsub message");
        match message {
            FloodsubEvent::Message(message) => println!(
                "Received: '{:?}' from {:?}",
                String::from_utf8_lossy(&message.data),
                message.source
            ),
            FloodsubEvent::Subscribed { peer_id, .. } => {
                log::debug!("Add peer to floodsub: {}", peer_id);
            }
            FloodsubEvent::Unsubscribed { peer_id, .. } => {
                log::debug!("Remove peer from floodsub: {}", peer_id);
            }
        }
    }
}

impl<TStore> NetworkBehaviourEventProcess<KademliaEvent> for Registry<TStore> {
    fn inject_event(&mut self, event: KademliaEvent) {
        match event {
            KademliaEvent::Discovered {
                peer_id,
                addresses,
                ty: _ty,
            } => {
                log::info!("Connected to: {:?} ({})", addresses, peer_id);
                log::info!("Adding to pub sub");
                self.floodsub.add_node_to_partial_view(peer_id);
                log::info!("Bootstrap was successful: {}", result.peer);
        }
    }
}
