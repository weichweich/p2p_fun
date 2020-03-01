use libp2p::{
    floodsub::{Floodsub, FloodsubEvent},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
pub struct Chat {
    pub floodsub: Floodsub,
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for Chat {
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
