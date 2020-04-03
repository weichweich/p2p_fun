use libp2p::{
    floodsub::{Floodsub, FloodsubEvent},
    kad::{Kademlia, KademliaEvent},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};
use log;

#[derive(NetworkBehaviour)]
pub struct Chat<TStore> {
    pub floodsub: Floodsub,
    pub kad: Kademlia<TStore>,
}

impl<TStore> NetworkBehaviourEventProcess<FloodsubEvent> for Chat<TStore> {
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

impl<TStore> NetworkBehaviourEventProcess<KademliaEvent> for Chat<TStore> {
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
            },
            KademliaEvent::UnroutablePeer {peer} => log::warn!("Unroutable peer! {}", peer),
            KademliaEvent::BootstrapResult(Ok(result)) => {
                log::info!("Bootstrap was successful: {}", result.peer);
            },
            other => log::trace!("Kademlia event: {:?}", other),
        }
    }
}
