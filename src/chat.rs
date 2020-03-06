use libp2p::{
    core::{connection::ConnectionId, ConnectedPoint},
    floodsub::{Floodsub, FloodsubEvent},
    swarm::{
        self, protocols_handler::DummyProtocolsHandler, NetworkBehaviourAction,
        NetworkBehaviourEventProcess, PollParameters, ProtocolsHandler,
    },
    NetworkBehaviour,
};
use std::task::Poll;

#[derive(NetworkBehaviour)]
pub struct Chat {
    pub floodsub: Floodsub,
    pub auto_subscribe: AutoSubscribe,
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

impl NetworkBehaviourEventProcess<ChatEvent> for Chat {
    fn inject_event(&mut self, message: ChatEvent) {}
}

pub enum ChatEvent {
    Message(String),
}

pub struct AutoSubscribe {}

impl swarm::NetworkBehaviour for AutoSubscribe {
    type ProtocolsHandler = DummyProtocolsHandler;
    type OutEvent = ChatEvent;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        Default::default()
    }

    fn addresses_of_peer(&mut self, peer_id: &libp2p::PeerId) -> Vec<libp2p::Multiaddr> {
        Vec::new()
    }

    fn inject_connected(&mut self, peer_id: libp2p::PeerId, endpoint: ConnectedPoint) {
    }

    fn inject_disconnected(&mut self, peer_id: &libp2p::PeerId, endpoint: ConnectedPoint) {}

    fn inject_event(
        &mut self,
        peer_id: libp2p::PeerId,
        connection: ConnectionId,
        event: void::Void,
    ) {
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context,
        params: &mut impl PollParameters,
    ) -> std::task::Poll<
        NetworkBehaviourAction<
            <Self::ProtocolsHandler as ProtocolsHandler>::InEvent,
            Self::OutEvent,
        >,
    > {
        Poll::Pending
    }
}
