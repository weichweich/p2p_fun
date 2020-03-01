use crate::{ARG_GEN, ARG_KEYPAIR};
use clap::ArgMatches;
use libp2p::identity::{ed25519, Keypair};
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
};

pub fn get_keypair(matches: &ArgMatches) -> Result<Keypair, Box<dyn Error>> {
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
