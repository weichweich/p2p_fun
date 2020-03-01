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
        if let FloodsubEvent::Message(message) = message {
            println!(
                "Received: '{:?}' from {:?}",
                String::from_utf8_lossy(&message.data),
                message.source
            );
        }
    }
}
