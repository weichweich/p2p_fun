use clap::{App, Arg, ArgMatches};
use libp2p::{
    floodsub::{self, Floodsub, FloodsubEvent},
    identity::{ed25519, Keypair},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour, PeerId, Swarm,
};
use log;
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
};
use stderrlog;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

const ARG_VERBOSE: &str = "verbose";
const ARG_KEYPAIR: &str = "keypair";
const ARG_GEN: &str = "keygen";
const ARG_TOPIC: &str = "topic";

const DEFAULT_TOPIC: &str = "chat";

fn get_keypair(matches: &ArgMatches) -> Result<Keypair, Box<dyn Error>> {
    if matches.occurrences_of(ARG_GEN) > 0 {
        log::info!("Generate new key pair.");
        let keypair = ed25519::Keypair::generate();

        // Write to file
        if let Some(filename) = matches.value_of(ARG_KEYPAIR) {
            log::info!("Write key to file");
            let mut file = File::create(filename)?;
            file.write_all(&keypair.encode()[..])?;
        }

        Ok(Keypair::Ed25519(keypair))
    } else if let Some(filename) = matches.value_of(ARG_KEYPAIR) {
        log::info!("Load key pair.");
        let mut keydata = Vec::new();
        let mut file = File::open(filename)?;
        file.read_to_end(&mut keydata)?;

        Ok(Keypair::Ed25519(ed25519::Keypair::decode(
            &mut keydata[..],
        )?))
    } else {
        let err_msg = format!(
            "Argument <{}> is required if <{}> is not set.",
            ARG_KEYPAIR, ARG_GEN
        );
        log::error!("{}", err_msg);

        Err(err_msg.into())
    }
}

#[derive(NetworkBehaviour)]
struct Chat {
    floodsub: Floodsub,

    // Struct fields which do not implement NetworkBehaviour need to be ignored
    #[behaviour(ignore)]
    #[allow(dead_code)]
    ignored_member: bool,
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

fn execute_app(matches: ArgMatches) -> Result<(), Box<dyn Error>> {
    // Generate new key pair
    let keypair = get_keypair(&matches)?;

    // Create PeerID
    let local_peer_id = PeerId::from(keypair.public());

    // Create transport protocol
    // TODO: use production transport
    let transport = libp2p::build_development_transport(keypair)?;

    let mut behaviour = Chat {
        floodsub: Floodsub::new(local_peer_id.clone()),
        ignored_member: false,
    };

    // Create a  floodsub topic
    let topic = matches.value_of(ARG_TOPIC).ok_or("Topic missing?")?;
    let floodsub_topic = floodsub::Topic::new(topic);
    log::info!("Topic is: {}", &topic);

    // Create a Swarm to manage peers and events
    behaviour.floodsub.subscribe(floodsub_topic);
    let mut _swarm = Swarm::new(transport, behaviour, local_peer_id);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let gen_arg_help = format!(
        "Generate a new key pair and write it to <{}>.",
        ARG_KEYPAIR
    );
    let app = App::new("Püür tö püür Fün")
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
            Arg::with_name(ARG_TOPIC)
                .short("t")
                .help("The topic to chat on.")
                .takes_value(true)
                .default_value(DEFAULT_TOPIC),
        );
    let matches = app.get_matches();

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
