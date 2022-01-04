//! A simple program which prints the animal name for a hotspot. It
//! will read the key from the ECC chip if present, otherwise it will
//! look for the keyfile in `/var/data/miner/swarm_key`

use angry_purple_tiger::AnimalName;
use ecc608_linux::{Ecc, KeyType};
use helium_crypto::{ecc_compact, Keypair, PublicKey};
use std::{convert::TryFrom, error::Error, fs};

fn main() {
    match go() {
        Ok(name) => print!("{}", name),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}

fn go() -> Result<String, Box<dyn std::error::Error>> {
    if !running_as_root() {
        return Err("must be root".into());
    }
    let pk = if let Ok(pk) = read_pk_from_ecc_chip() {
        pk
    } else {
        read_pk_from_swarmkey_file()?
    };
    let pk_string = pk.to_string();
    Ok(pk_string.parse::<AnimalName>()?.to_string())
}

#[cfg(unix)]
fn running_as_root() -> bool {
    let euid = unsafe { libc::geteuid() };
    euid == 0
}

fn read_pk_from_ecc_chip() -> Result<PublicKey, Box<dyn Error>> {
    const ECC_I2C_PATH: &str = "/dev/i2c-1";
    const ECC_I2C_ADDR: u16 = 0x60;
    const KEY_SLOT: u8 = 0;

    let mut ecc = Ecc::from_path(ECC_I2C_PATH, ECC_I2C_ADDR)?;
    // Start with the "decompressed" sec1 tag since the ecc does not include it.
    let mut key_bytes = vec![4u8];
    // Add the keybytes from the slot.
    key_bytes.extend_from_slice(ecc.genkey(KeyType::Public, KEY_SLOT)?.as_ref());
    let public_key = PublicKey::from(ecc_compact::PublicKey::try_from(key_bytes.as_ref())?);
    Ok(public_key)
}

fn read_pk_from_swarmkey_file() -> Result<PublicKey, Box<dyn Error>> {
    const BACKUP_KEY_FILE: &str = "/var/data/miner/swarm_key";
    let key_pair_bytes = fs::read(BACKUP_KEY_FILE)?;
    let key_pair = Keypair::try_from(&key_pair_bytes[..33])?;
    Ok(key_pair.public_key().clone())
}
