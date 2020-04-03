#![deny(unsafe_code)]
use async_std::{io, task};
use futures::prelude::*;
use libp2p::Transport;
use libp2p::{
    core, dns,
    floodsub::{self, Floodsub},
    identity, kad, mplex, secio, tcp, yamux, Multiaddr, PeerId, Swarm,
};
use log;
use std::{
    error::{self, Error},
    task::{Context, Poll}, str::FromStr,
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

fn build_transport(
    keypair: identity::Keypair,
) -> Result<
    impl Transport<
            Output = (
                PeerId,
                impl core::muxing::StreamMuxer<
                        OutboundSubstream = impl Send,
                        Substream = impl Send,
                        Error = impl Into<io::Error>,
                    > + Send
                    + Sync,
            ),
            Error = impl error::Error + Send,
            Listener = impl Send,
            Dial = impl Send,
            ListenerUpgrade = impl Send,
        > + Clone,
    Box<dyn Error>,
> {
    let tcp_cfg = tcp::TcpConfig::new();
    let dns_cfg = dns::DnsConfig::new(tcp_cfg)?;

    let transport = dns_cfg
        .upgrade(core::upgrade::Version::V1)
        .authenticate(secio::SecioConfig::new(keypair))
        .multiplex(core::upgrade::SelectUpgrade::new(
            yamux::Config::default(),
            mplex::MplexConfig::new(),
        ))
        .map(|(peer, muxer), _| (peer, core::muxing::StreamMuxerBox::new(muxer)));
    Ok(transport)
}

fn without_first(string: &str) -> &str {
    string
        .char_indices()
        .nth(1)
        .and_then(|(i, _)| string.get(i..))
        .unwrap_or("")
}

fn parse_address(raw_address: &str) -> Result<(Multiaddr, PeerId), Box<dyn Error>> {
    // find index of last '/' which should separate the multiaddress from the PeerID
    let split_i = raw_address.rfind("/").ok_or("Invalid address")?;
    // split the multiaddress from the PeerID
    let (raw_addr, raw_id_padded) = raw_address.split_at(split_i);
    let raw_id = without_first(raw_id_padded);

    // parse multiaddress and PeerID
    let addr: Multiaddr = raw_addr.parse()?;
    let peer_id = PeerId::from_str(raw_id)?;

    Ok((addr, peer_id))
}

fn execute_app(matches: Opt) -> Result<(), Box<dyn Error>> {
    // Generate new key pair
    let keypair = key::get_keypair(&matches)?;

    // Create PeerID
    let local_peer_id = PeerId::from(keypair.public());
    println!("Own peer ID: {}", local_peer_id.to_base58());

    // Create transport protocol
    let transport = build_transport(keypair)?;

    // Create behaviors
    let kad_cfg = kad::KademliaConfig::default();
    let kad_store = kad::record::store::MemoryStore::new(local_peer_id.clone());
    let kad = kad::Kademlia::with_config(local_peer_id.clone(), kad_store, kad_cfg);
    let mut behaviour = chat::Chat {
        floodsub: Floodsub::new(local_peer_id.clone()),
        kad,
    };

    // Create a  floodsub topic
    let floodsub_topic = floodsub::Topic::new(&matches.topic);

    // Reach out to another node if specified
    if let Some(peers) = &matches.peers {
        for peer in peers {
            let (addr, peer_id) = parse_address(peer)?;
            behaviour.kad.add_address(&peer_id, addr);
        }
    }

    behaviour.kad.bootstrap();

    // Create a Swarm to manage peers and events
    let mut swarm = Swarm::new(transport, behaviour, local_peer_id);

    // Listen on all interfaces and whatever port the OS assigns
    Swarm::listen_on(&mut swarm, "/ip6/::/tcp/0".parse()?)?;

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Kick it off
    // TODO: use async_std and await syntax
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
