use clap::{App, Arg, SubCommand};
use hex;
use libp2p::identity::ed25519::Keypair;
use log::{info, warn};
use stderrlog;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

const ARG_VERBOSE: &str = "verbose";

fn main() {
    let matches = App::new("QR Master")
        .version(VERSION)
        .author(AUTHOR)
        .about(DESCRIPTION)
        .arg(
            Arg::with_name(ARG_VERBOSE)
                .short("v")
                .help("Sets the level of verbosity")
                .multiple(true)
                .takes_value(false),
        )
        .get_matches();

    stderrlog::new()
        .verbosity(matches.occurrences_of(ARG_VERBOSE) as usize)
        .init()
        .unwrap();
    let keypair = Keypair::generate();
    info!("generated ed25519 keypair");
    {
        let bytes = keypair.encode();
        let pretty_output = hex::encode(&bytes[..]);
        info!("Keypair: {:?}", pretty_output);
    }
}
