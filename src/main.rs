use async_std::{io, task};
use clap::{App, Arg, ArgMatches};
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

mod chat;
mod key;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

const ARG_VERBOSE: &str = "verbose";
const ARG_KEYPAIR: &str = "keypair";
const ARG_GEN: &str = "keygen";
const ARG_TOPIC: &str = "topic";
const ARG_PEER: &str = "peer";

const DEFAULT_TOPIC: &str = "chat";

fn execute_app(matches: ArgMatches) -> Result<(), Box<dyn Error>> {
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
    };

    // Create a  floodsub topic
    let topic = matches.value_of(ARG_TOPIC).ok_or("Topic missing?")?;
    let floodsub_topic = floodsub::Topic::new(topic);

    // Create a Swarm to manage peers and events
    let mut swarm = Swarm::new(transport, behaviour, local_peer_id);

    // Reach out to another node if specified
    if let Some(peers) = matches.values_of(ARG_PEER) {
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
                        log::info!("subscribed to: {}", &topic);
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
    let gen_arg_help = format!("Generate a new key pair and write it to <{}>.", ARG_KEYPAIR);
    let matches = App::new("Püür tö püür Fün")
        .version(VERSION)
        .author(AUTHOR)
        .about(DESCRIPTION)
        .arg(
            Arg::with_name(ARG_VERBOSE)
                .short("v")
                .help("Sets the level of verbosity.")
                .multiple(true)
                .takes_value(false),
        )
        .arg(
            Arg::with_name(ARG_KEYPAIR)
                .short("k")
                .help("File containing the key pair.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(ARG_GEN)
                .short("g")
                .takes_value(false)
                .help(&gen_arg_help),
        )
        .arg(
            Arg::with_name(ARG_PEER)
                .short("p")
                .takes_value(true)
                .multiple(true)
                .help("Address of a peer to connect to."),
        )
        .arg(
            Arg::with_name(ARG_TOPIC)
                .short("t")
                .help("The topic to chat on.")
                .takes_value(true)
                .default_value(DEFAULT_TOPIC),
        )
        .get_matches();

    // Configure logger
    stderrlog::new()
        .verbosity(matches.occurrences_of(ARG_VERBOSE) as usize)
        .init()?;

    if let Err(box_err) = execute_app(matches) {
        eprintln!("Application error:\n{}", box_err);
        Err(box_err)
    } else {
        Ok(())
    }
}
