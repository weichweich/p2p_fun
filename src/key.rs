use crate::Opt;
use libp2p::identity::{ed25519, Keypair};
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
};

pub fn get_keypair(matches: &Opt) -> Result<Keypair, Box<dyn Error>> {
    if matches.generate_key {
        log::info!("Generate new key pair.");
        let keypair = ed25519::Keypair::generate();

        // Write to file
        if let Some(filename) = &matches.keypair {
            log::info!("Write key to file");
            let mut file = File::create(filename)?;
            file.write_all(&keypair.encode()[..])?;
        }

        Ok(Keypair::Ed25519(keypair))
    } else if let Some(filename) = &matches.keypair {
        log::info!("Load key pair.");
        let mut keydata = Vec::new();
        let mut file = File::open(filename)?;
        file.read_to_end(&mut keydata)?;

        Ok(Keypair::Ed25519(ed25519::Keypair::decode(
            &mut keydata[..],
        )?))
    } else {
        let err_msg = "Argument <KEYPAIR> is required if <generate_key> is not set.";
        log::error!("{}", err_msg);

        Err(err_msg.into())
    }
}
