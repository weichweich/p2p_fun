#![deny(unsafe_code)]
use async_std::{io, task};
use chat::AutoSubscribe;
use futures::prelude::*;
use libp2p::{
    floodsub::{self, Floodsub},
    Multiaddr, PeerId, Swarm,
};
use log;
use std::{
    error::Error,
    task::{Context, Poll},
};
use stderrlog;
use structopt::StructOpt;

mod chat;
mod key;

const NAME: &str = env!("CARGO_PKG_NAME");

const DEFAULT_TOPIC: &str = "chat";

#[derive(Debug, StructOpt)]
#[structopt(name = NAME)]
pub struct Opt {
    #[structopt(
        short,
        long,
        parse(from_occurrences),
        help = "Sets the level of verbosity."
    )]
    verbose: u8,

    #[structopt(short, long, help = "File containing the key pair.")]
    keypair: Option<String>,

    #[structopt(
        short,
        long,
        required_if("keypair", ""),
        help = "Generate a new key pair and write it to <KEYPAIR>."
    )]
    generate_key: bool,

    #[structopt(short, long, default_value = DEFAULT_TOPIC, help = "The topic to chat on.")]
    topic: String,

    #[structopt(short, long, help = "Address of a peer to connect to.")]
    peers: Option<Vec<String>>,
}

fn execute_app(matches: Opt) -> Result<(), Box<dyn Error>> {
    // Generate new key pair
    let keypair = key::get_keypair(&matches)?;

    // Create PeerID
    let local_peer_id = PeerId::from(keypair.public());
    println!("Own peer ID: {}", local_peer_id);

    // Create transport protocol
    // TODO: use production transport
    let transport = libp2p::build_development_transport(keypair)?;

    let behaviour = chat::Chat {
        floodsub: Floodsub::new(local_peer_id.clone()),
        auto_subscribe: AutoSubscribe {},
    };

    // Create a  floodsub topic
    let floodsub_topic = floodsub::Topic::new(&matches.topic);

    // Create a Swarm to manage peers and events
    let mut swarm = Swarm::new(transport, behaviour, local_peer_id);

    // Reach out to another node if specified
    if let Some(peers) = &matches.peers {
        for peer in peers {
            let addr: Multiaddr = peer.parse()?;
            Swarm::dial_addr(&mut swarm, addr)?;
            log::info!("Dialed {:?}", peer);
        }
    }

    // Listen on all interfaces and whatever port the OS assigns
    Swarm::listen_on(&mut swarm, "/ip6/::/tcp/0".parse()?)?;

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Kick it off
    let mut listening = false;
    let mut subscribed = false;
    task::block_on(future::poll_fn(move |cx: &mut Context| {
        loop {
            match stdin.try_poll_next_unpin(cx)? {
                Poll::Ready(Some(line)) => {
                    // now that we are connected to peers, subscribe to a topic.
                    if !subscribed && swarm.floodsub.subscribe(floodsub_topic.clone()) {
                        log::info!("subscribed to: {}", &matches.topic);
                        subscribed = true;
                    }
                    swarm
                        .floodsub
                        .publish(floodsub_topic.clone(), line.as_bytes());
                    log::info!("sent");
                }
                Poll::Ready(None) => panic!("Stdin closed"),
                Poll::Pending => break,
            }
        }
        loop {
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(event)) => log::info!("New Event: {:?}", event),
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Pending => {
                    if !listening {
                        for addr in Swarm::listeners(&swarm) {
                            println!("Listening on {:?}", addr);
                            listening = true;
                        }
                    }
                    break;
                }
            }
        }
        Poll::Pending
    }))
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    // Configure logger
    stderrlog::new().verbosity(opt.verbose as usize).init()?;

    if let Err(box_err) = execute_app(opt) {
        eprintln!("Application error:\n{}", box_err);
        Err(box_err)
    } else {
        Ok(())
    }
}
