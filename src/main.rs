use clap::{App, Arg, ArgMatches};
use libp2p::{
    identity::{Keypair, ed25519},
    tcp::TcpConfig, PeerId, Multiaddr, Transport,
};
use log;
use stderrlog;
use std::fs::File;
use std::io::{Read, Write};
use std::error::Error;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

const ARG_VERBOSE: &str = "verbose";
const ARG_KEYPAIR: &str = "keypair";
const ARG_GEN: &str = "keygen";

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

        Ok(Keypair::Ed25519(ed25519::Keypair::decode(&mut keydata[..])?))
    } else {
        let err_msg = format!(
            "Argument <{}> is required if <{}> is not set.",
            ARG_KEYPAIR, ARG_GEN
        );
        log::error!("{}", err_msg);

        Err(err_msg.into())
    }
}

fn main() {
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
                .help(&format!(
                    "Generate a new key pair and write it to <{}>.",
                    ARG_KEYPAIR
                )),
        )
        .get_matches();

    // Configure logger
    stderrlog::new()
        .verbosity(matches.occurrences_of(ARG_VERBOSE) as usize)
        .init()
        .unwrap();

    // Generate new key pair
    let keypair = get_keypair(&matches).unwrap();

    // Create PeerID
    let local_peer_id = PeerId::from(keypair.public());

    // Create transport protocol
    let transport = TcpConfig::new();
    let addr: Multiaddr = "/ip6/98.97.96.95/tcp/0".parse().expect("invalid multiaddr");
    let _conn = transport.dial(addr);
}
