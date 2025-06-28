use crate::cli::{Args, Parser};
use libp2p::{PeerId, identity::Keypair};
use sha3::{Digest, Keccak256};
use std::error::Error;

pub fn ed25519_from_seed(seed: &String) -> Result<Keypair, Box<dyn Error>> {
  let mut hasher = Keccak256::new();
  hasher.update(seed.as_bytes());
  let bytes = hasher.finalize();
  let keypair = Keypair::ed25519_from_bytes(bytes)?;
  Ok(keypair)
}

/// Create a random key for ourselves & read user's inputs
pub fn read_keypair() -> Result<Keypair, Box<dyn Error>> {
  let Args { seed, .. } = Args::parse();
  let keypair = if let Some(s) = seed {
    ed25519_from_seed(&s)?
  } else {
    Keypair::generate_ed25519()
  };
  Ok(keypair)
}

/// Extract peer id from multiaddr
pub fn parse_peer_id(addr: &String) -> Result<PeerId, Box<dyn Error>> {
  let parts: Vec<&str> = addr.split("/p2p/").collect();
  let str = parts.last().copied().ok_or("Cannot parse peer id.")?;
  let buf = bs58::decode(str).into_vec()?;
  let id = PeerId::from_bytes(&buf)?;
  Ok(id)
}
